use super::core::{FontAtlas, InstanceData, Vertex};
use wgpu::util::DeviceExt;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const PICKING_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const WBOIT_ACCUMULATION_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const WBOIT_REVEALAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
pub const USAGE_BINDING: wgpu::TextureUsages =
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING);
pub const USAGE_COPY_SRC: wgpu::TextureUsages =
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::COPY_SRC);

pub struct Renderer {
    pub pipeline: wgpu::RenderPipeline,
    pub picking_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub depth_texture_view: wgpu::TextureView,

    pub picking_texture: wgpu::Texture,
    pub picking_texture_view: wgpu::TextureView,
    pub picking_depth_texture_view: wgpu::TextureView,
    pub picking_staging_buffer: wgpu::Buffer,

    // WBOIT (Weighted Blended Order-Independent Transparency)
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub composite_pipeline: wgpu::RenderPipeline,
    pub wboit_accumulation_texture_view: wgpu::TextureView,
    pub wboit_revealage_texture_view: wgpu::TextureView,
    pub wboit_bind_group: wgpu::BindGroup,

    pub font_atlas_texture: wgpu::Texture,
    pub font_atlas_texture_view: wgpu::TextureView,
    pub font_atlas_sampler: wgpu::Sampler,

    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        font_atlas: &FontAtlas,
    ) -> Self {
        // Create shader modules
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/main.wgsl").into()),
        });

        let wboit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("WBOIT Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wboit.wgsl").into()),
        });

        // Create uniform buffer for 4 matrices (256 bytes) + 3 u32 flags (8 bytes) + padding (8 bytes)
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: &[0u8; 272],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create render pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            "Render Pipeline",
            &shader,
            "vs_main",
            "fs_main",
            config.format,
        );
        let picking_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            "Picking Pipeline",
            &shader,
            "vs_main",
            "fs_main",
            PICKING_FORMAT,
        );
        let transparent_pipeline = Self::create_transparent_pipeline(device, &pipeline_layout, &shader);

        // Create WBOIT textures
        let (_, depth_texture_view) =
            Self::create_texture(device, config, "Depth Texture", DEPTH_FORMAT, USAGE_BINDING);
        let (picking_texture, picking_texture_view) =
            Self::create_texture(device, config, "Picking Texture", PICKING_FORMAT, USAGE_COPY_SRC);
        let (_, picking_depth_texture_view) =
            Self::create_texture(device, config, "Picking Depth Texture", DEPTH_FORMAT, USAGE_BINDING);
        let picking_staging_buffer = Self::create_picking_staging_buffer(device);

        let (_, wboit_accumulation_texture_view) = Self::create_texture(
            device,
            config,
            "WBOIT Accum Texture",
            WBOIT_ACCUMULATION_FORMAT,
            USAGE_BINDING,
        );
        let (_, wboit_revealage_texture_view) = Self::create_texture(
            device,
            config,
            "WBOIT Reveal Texture",
            WBOIT_REVEALAGE_FORMAT,
            USAGE_BINDING,
        );

        // Create WBOIT composite pipeline and bind group
        let (composite_pipeline, wboit_bind_group_layout) =
            Self::create_composite_pipeline(device, &wboit_shader, config.format);

        let wboit_bind_group = Self::create_wboit_bind_group(
            device,
            &wboit_bind_group_layout,
            &wboit_accumulation_texture_view,
            &wboit_revealage_texture_view,
        );

        let (font_atlas_texture, font_atlas_texture_view, font_atlas_sampler) =
            Self::create_font_atlas_texture(device, queue, font_atlas);

        // Create bind group (after font atlas texture is created)
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&font_atlas_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&font_atlas_sampler),
                },
            ],
        });

        Self {
            pipeline,
            picking_pipeline,
            transparent_pipeline,
            composite_pipeline,
            uniform_buffer,
            bind_group,
            depth_texture_view,
            picking_texture,
            picking_texture_view,
            picking_depth_texture_view,
            picking_staging_buffer,
            wboit_accumulation_texture_view,
            wboit_revealage_texture_view,
            wboit_bind_group,
            font_atlas_texture,
            font_atlas_texture_view,
            font_atlas_sampler,
            width: config.width,
            height: config.height,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        (_, self.depth_texture_view) =
            Self::create_texture(device, config, "Depth Texture", DEPTH_FORMAT, USAGE_BINDING);
        let (picking_texture, picking_texture_view) =
            Self::create_texture(device, config, "Picking Texture", PICKING_FORMAT, USAGE_COPY_SRC);
        self.picking_texture = picking_texture;
        self.picking_texture_view = picking_texture_view;
        (_, self.picking_depth_texture_view) =
            Self::create_texture(device, config, "Picking Depth Texture", DEPTH_FORMAT, USAGE_BINDING);

        // Recreate WBOIT textures
        let (_, wboit_accumulation_texture_view) = Self::create_texture(
            device,
            config,
            "WBOIT Accum Texture",
            WBOIT_ACCUMULATION_FORMAT,
            USAGE_BINDING,
        );
        let (_, wboit_revealage_texture_view) = Self::create_texture(
            device,
            config,
            "WBOIT Reveal Texture",
            WBOIT_REVEALAGE_FORMAT,
            USAGE_BINDING,
        );
        self.wboit_accumulation_texture_view = wboit_accumulation_texture_view;
        self.wboit_revealage_texture_view = wboit_revealage_texture_view;

        // Recreate WBOIT bind group with new textures
        let wboit_bind_group_layout = Self::create_wboit_bind_group_layout(device);
        self.wboit_bind_group = Self::create_wboit_bind_group(
            device,
            &wboit_bind_group_layout,
            &self.wboit_accumulation_texture_view,
            &self.wboit_revealage_texture_view,
        );

        self.width = config.width;
        self.height = config.height;
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        label: &str,
        shader: &wgpu::ShaderModule,
        vertex_entry_point: &str,
        fragment_entry_point: &str,
        fragment_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some(vertex_entry_point),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fragment_entry_point),
                targets: &[Some(wgpu::ColorTargetState {
                    format: fragment_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    fn create_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format,
            usage: usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn create_picking_staging_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        // Buffer for reading a single pixel (4 bytes RGBA)
        // Must be aligned to 256 bytes for COPY_DST
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Picking Staging Buffer"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    fn create_transparent_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Transparent Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_transparent"),
                targets: &[
                    // Accumulation target: additive blending (ONE, ONE)
                    Some(wgpu::ColorTargetState {
                        format: WBOIT_ACCUMULATION_FORMAT,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    // Revealage target: multiplicative blending (ZERO, ONE_MINUS_SRC)
                    Some(wgpu::ColorTargetState {
                        format: WBOIT_REVEALAGE_FORMAT,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero,
                                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for transparent objects
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: false,                 // Don't write to depth buffer
                depth_compare: wgpu::CompareFunction::Less, // But still test against it
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    fn create_wboit_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("WBOIT Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_composite_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let bind_group_layout = Self::create_wboit_bind_group_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("WBOIT Composite Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("WBOIT Composite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        (pipeline, bind_group_layout)
    }

    fn create_wboit_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        accumulation_view: &wgpu::TextureView,
        revealage_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WBOIT Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(accumulation_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(revealage_view),
                },
            ],
        })
    }

    fn create_font_atlas_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_atlas: &FontAtlas,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let size = font_atlas.size;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Font Atlas Texture"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &font_atlas.texture,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Font Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        (texture, view, sampler)
    }
}
