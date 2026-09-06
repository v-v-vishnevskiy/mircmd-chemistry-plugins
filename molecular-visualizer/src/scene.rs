use shared_lib::types::{AtomicCoordinates, VolumeCube as VolumeCubeData};

use super::atom::AtomInfo;
use super::config::Config;
use super::coordinate_axes::CoordinateAxes;
use super::core::{Camera, FontAtlas, Mat4, ProjectionManager, ProjectionMode, Transform, Vec3, mesh_objects};
use super::image_export;
use super::molecule::Molecule;
use super::renderer::Renderer;
use super::types::Color;
use super::utils::color_to_id;
use super::vertex_buffer::VertexBuffer;
use super::volume_cube::VolumeCube;

pub struct Scene {
    pub projection_manager: ProjectionManager,
    pub transform: Transform,
    pub renderer: Renderer,

    camera: Camera,
    molecule: Option<Molecule>,
    pub volume_cube: Option<VolumeCube>,
    pub coordinate_axes: CoordinateAxes,
    font_atlas: FontAtlas,
    rect_vb: VertexBuffer, // for character rendering

    picking_texture_dirty: bool,
}

impl Scene {
    pub fn has_molecule(&self) -> bool {
        self.molecule.is_some()
    }

    pub fn molecule_max_coordinate(&self) -> Option<f32> {
        self.molecule.as_ref().map(|m| m.max_coordinate())
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let font_atlas = FontAtlas::from_embedded_font(4096, 550.0, 3);
        let rect_vb = Self::create_font_atlas_vb(device);

        let coordinate_axes = CoordinateAxes::new(device);

        Self {
            projection_manager: ProjectionManager::new(1, 1, ProjectionMode::Perspective),
            transform: Transform::new(),
            renderer: Renderer::new(device, queue, surface_config, &font_atlas),
            camera: Camera::new(),
            molecule: None,
            volume_cube: None,
            coordinate_axes,
            font_atlas,
            rect_vb,
            picking_texture_dirty: true,
        }
    }

    fn create_font_atlas_vb(device: &wgpu::Device) -> VertexBuffer {
        let mut rect = mesh_objects::rect::create(-1.0, 1.0, -1.0, 1.0);
        rect.vertices[0].tex_coord = [0.0, 0.0];
        rect.vertices[1].tex_coord = [1.0, 0.0];
        rect.vertices[2].tex_coord = [1.0, 1.0];
        rect.vertices[3].tex_coord = [0.0, 1.0];
        VertexBuffer::new(device, &rect)
    }

    fn setup_camera(&mut self, scene_size: f32) {
        self.projection_manager
            .orthographic_projection
            .set_view_bounds(scene_size + scene_size * 0.10);

        let fov_factor = self.projection_manager.perspective_projection.get_fov() / 45.0;
        self.projection_manager
            .perspective_projection
            .set_near_far_plane(0.1, 10.0 * scene_size / fov_factor);

        self.camera.reset_to_default();
        self.camera.set_position(Vec3::new(0.0, 0.0, 3.0 * scene_size));
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.renderer.resize(device, config);
    }

    pub fn load_atomic_coordinates(&mut self, device: &wgpu::Device, config: &Config, data: AtomicCoordinates) {
        match Molecule::new(device, config, data, &self.font_atlas) {
            Ok(molecule) => {
                self.setup_camera(molecule.radius);
                self.coordinate_axes.set_position(molecule.center);
                self.molecule = Some(molecule);
            }
            Err(_) => {}
        }
    }

    pub fn load_volume_cube(&mut self, data: VolumeCubeData) {
        self.volume_cube = Some(VolumeCube::new(data));
    }

    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &Config,
        render_mode: u32,
    ) {
        let surface_texture = match surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(_) => return,
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let clear = wgpu::Color {
            r: config.style.background_color.r as f64,
            g: config.style.background_color.g as f64,
            b: config.style.background_color.b as f64,
            a: 1.0,
        };
        self.encode_and_submit(&view, device, queue, config, render_mode, clear);
        surface_texture.present();
        self.picking_texture_dirty = true;
    }

    fn encode_and_submit(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &Config,
        render_mode: u32,
        clear: wgpu::Color,
    ) {
        let molecule_transform = match &self.molecule {
            Some(molecule) => molecule.transform,
            None => Mat4::new(),
        };
        let projection_matrix = *self.projection_manager.get_matrix();
        let view_matrix = *self.camera.get_matrix();
        let scene_matrix = *self.transform.get_matrix() * molecule_transform;
        let final_matrix = projection_matrix * view_matrix * scene_matrix;
        let is_perspective = self.projection_manager.mode == ProjectionMode::Perspective;
        let mut uniforms_data = [0u8; 272];
        uniforms_data[0..64].copy_from_slice(bytemuck::cast_slice(&projection_matrix.data));
        uniforms_data[64..128].copy_from_slice(bytemuck::cast_slice(&view_matrix.data));
        uniforms_data[128..192].copy_from_slice(bytemuck::cast_slice(&scene_matrix.data));
        uniforms_data[192..256].copy_from_slice(bytemuck::cast_slice(&final_matrix.data));
        uniforms_data[256..260].copy_from_slice(&render_mode.to_le_bytes());
        uniforms_data[260..264].copy_from_slice(&(if is_perspective { 1u32 } else { 0u32 }).to_le_bytes());
        queue.write_buffer(&self.renderer.uniform_buffer, 0, &uniforms_data);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        self.coordinate_axes.update(device, &self.font_atlas);
        self.encode_opaque_pass(&mut encoder, view, config, clear);
        self.encode_transparent_pass(&mut encoder);
        self.encode_composite_pass(&mut encoder, view);
        queue.submit(std::iter::once(encoder.finish()));
    }

    fn encode_opaque_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _config: &Config,
        clear: wgpu::Color,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Opaque Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.renderer.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&self.renderer.pipeline);
        render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);
        if let Some(obj) = &self.molecule {
            obj.render_atoms_and_bonds(&mut render_pass);
        }
        if let Some(obj) = &self.volume_cube {
            obj.render_opaque(&mut render_pass);
        }
        self.coordinate_axes.render_axes(&mut render_pass);
    }

    fn encode_transparent_pass(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("WBOIT Transparent Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.renderer.wboit_accumulation_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.renderer.wboit_revealage_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.renderer.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);
        if let Some(obj) = &self.molecule {
            render_pass.set_pipeline(&self.renderer.transparent_pipeline);
            obj.render_bounding_spheres(&mut render_pass);
            render_pass.set_pipeline(&self.renderer.text_transparent_pipeline);
            render_pass.set_vertex_buffer(0, self.rect_vb.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.rect_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            obj.render_labels(&mut render_pass, &self.rect_vb);
        }
        if let Some(obj) = &self.volume_cube {
            render_pass.set_pipeline(&self.renderer.transparent_pipeline);
            obj.render_transparent(&mut render_pass);
        }
        render_pass.set_pipeline(&self.renderer.text_transparent_pipeline);
        render_pass.set_vertex_buffer(0, self.rect_vb.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.rect_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.coordinate_axes.render_labels(&mut render_pass, &self.rect_vb);
    }

    fn encode_composite_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("WBOIT Composite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&self.renderer.composite_pipeline);
        render_pass.set_bind_group(0, &self.renderer.wboit_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn render_picking_pass(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let molecule_transform = match &self.molecule {
            Some(molecule) => molecule.transform,
            None => Mat4::new(),
        };

        // Calculate matrices (same as main render)
        let projection_matrix = *self.projection_manager.get_matrix();
        let view_matrix = *self.camera.get_matrix();
        let scene_matrix = *self.transform.get_matrix() * molecule_transform;
        let final_matrix = projection_matrix * view_matrix * scene_matrix;
        let is_perspective = self.projection_manager.mode == ProjectionMode::Perspective;
        let render_mode = 1u32; // Picking mode
        let lighting_model = 0u32; // No lighting for picking

        let mut uniforms_data = [0u8; 272];
        uniforms_data[0..64].copy_from_slice(bytemuck::cast_slice(&projection_matrix.data));
        uniforms_data[64..128].copy_from_slice(bytemuck::cast_slice(&view_matrix.data));
        uniforms_data[128..192].copy_from_slice(bytemuck::cast_slice(&scene_matrix.data));
        uniforms_data[192..256].copy_from_slice(bytemuck::cast_slice(&final_matrix.data));
        uniforms_data[256..260].copy_from_slice(&render_mode.to_le_bytes());
        uniforms_data[260..264].copy_from_slice(&(if is_perspective { 1u32 } else { 0u32 }).to_le_bytes());
        uniforms_data[264..268].copy_from_slice(&lighting_model.to_le_bytes());

        queue.write_buffer(&self.renderer.uniform_buffer, 0, &uniforms_data);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Picking Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Picking Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.renderer.picking_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.picking_depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.renderer.picking_pipeline);

            match &self.molecule {
                Some(molecule) => {
                    render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);
                    molecule.render_picking_frame(&mut render_pass);
                }
                None => {}
            };
        }

        queue.submit(std::iter::once(encoder.finish()));
        self.picking_texture_dirty = false;
    }

    pub fn submit_pick(
        &mut self,
        x: u32,
        y: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<PendingPickMap> {
        if self.molecule.is_none() {
            return None;
        }
        let (width, height) = self.renderer.get_size();
        if x >= width || y >= height {
            return None;
        }
        if self.picking_texture_dirty {
            self.render_picking_pass(device, queue);
        }
        let staging = create_readback_buffer(device, 256, "Pick Staging Buffer");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Picking Read Encoder"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.renderer.picking_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        Some(PendingPickMap {
            staging,
            device: device.clone(),
        })
    }

    pub fn apply_hover(
        &mut self,
        atom_index: usize,
        device: &wgpu::Device,
        config: &Config,
    ) -> (Option<AtomInfo>, bool) {
        let Some(molecule) = self.molecule.as_mut() else {
            return (None, false);
        };
        molecule.highlight_atom(atom_index, device, &self.font_atlas, config)
    }

    pub fn apply_selection(&mut self, atom_index: usize, device: &wgpu::Device, config: &Config) -> bool {
        let Some(molecule) = self.molecule.as_mut() else {
            return false;
        };
        molecule.toggle_atom_selection(atom_index, device, &self.font_atlas, config)
    }

    pub fn toggle_projection(&mut self) {
        self.projection_manager.toggle_projection_mode();
    }

    pub fn set_atom_labels_symbol_visible(&mut self, value: bool) {
        if self.molecule.is_none() {
            return;
        }
        self.molecule.as_mut().unwrap().set_atom_labels_symbol_visible(value);
    }

    pub fn set_atom_labels_number_visible(&mut self, value: bool) {
        if self.molecule.is_none() {
            return;
        }
        self.molecule.as_mut().unwrap().set_atom_labels_number_visible(value);
    }

    pub fn set_all_atom_labels_visible(&mut self, value: bool) {
        if self.molecule.is_none() {
            return;
        }
        self.molecule.as_mut().unwrap().set_all_atom_labels_visible(value);
    }

    pub fn set_selected_atom_labels_visible(&mut self, value: bool) {
        if self.molecule.is_none() {
            return;
        }
        self.molecule.as_mut().unwrap().set_selected_atom_labels_visible(value);
    }

    pub fn toggle_all_atom_labels_visible(&mut self) -> bool {
        if self.molecule.is_none() {
            return false;
        }
        self.molecule.as_mut().unwrap().toggle_all_atom_labels_visible()
    }

    pub fn toggle_selected_atom_labels_visible(&mut self) {
        if self.molecule.is_none() {
            return;
        }
        self.molecule.as_mut().unwrap().toggle_selected_atom_labels_visible();
    }

    pub fn update_atom_labels(&mut self, device: &wgpu::Device, config: &Config) {
        if self.molecule.is_none() {
            return;
        }
        self.molecule
            .as_mut()
            .unwrap()
            .update_atom_labels(device, &self.font_atlas, config);
    }

    pub fn apply_style(&mut self, device: &wgpu::Device, config: &Config) {
        if let Some(molecule) = self.molecule.as_mut() {
            molecule.apply_style(device, &self.font_atlas, config);
        }
    }

    pub fn submit_export(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &Config,
        surface_config: &wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
        background: Color,
        crop: bool,
    ) -> Result<PendingImageMap, String> {
        validate_export_size(width, height)?;
        let previous = (surface_config.width, surface_config.height);
        self.resize_for_export(device, surface_config, width, height);
        let (texture, view) = Renderer::create_color_target(device, width, height, surface_config.format);
        let clear = wgpu::Color {
            r: background.r as f64,
            g: background.g as f64,
            b: background.b as f64,
            a: background.a as f64,
        };
        self.encode_and_submit(&view, device, queue, config, 0, clear);
        let bytes_per_row = image_export::padded_bytes_per_row(width);
        let staging = create_readback_buffer(device, bytes_per_row as u64 * height as u64, "Export Staging Buffer");
        copy_texture_to_buffer(device, queue, &texture, &staging, width, height, bytes_per_row);
        self.resize_for_export(device, surface_config, previous.0, previous.1);
        Ok(PendingImageMap {
            staging,
            device: device.clone(),
            width,
            height,
            bytes_per_row,
            swizzle_bgra: image_export::is_bgra(surface_config.format),
            crop,
            background: image_export::color_to_u8(background),
        })
    }

    fn resize_for_export(
        &mut self,
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
    ) {
        let mut config = surface_config.clone();
        config.width = width.max(1);
        config.height = height.max(1);
        self.resize(device, &config);
        self.projection_manager.set_viewport(config.width, config.height);
    }

    pub fn update_coordinate_axes(&mut self, device: &wgpu::Device) {
        self.coordinate_axes.update(device, &self.font_atlas);
    }
}

pub struct PendingPickMap {
    staging: wgpu::Buffer,
    device: wgpu::Device,
}

pub struct PendingImageMap {
    staging: wgpu::Buffer,
    device: wgpu::Device,
    width: u32,
    height: u32,
    bytes_per_row: u32,
    swizzle_bgra: bool,
    crop: bool,
    background: [u8; 4],
}

pub async fn finish_pick(pending: PendingPickMap) -> usize {
    match map_buffer(&pending.staging, &pending.device, 0..4u64).await {
        Ok(data) => color_to_id(data[0], data[1], data[2]),
        Err(_) => 0,
    }
}

pub async fn finish_export(pending: PendingImageMap) -> Result<(u32, u32, Vec<u8>), String> {
    let range = 0..(pending.bytes_per_row as u64 * pending.height as u64);
    let mapped = map_buffer(&pending.staging, &pending.device, range).await?;
    let mut pixels = image_export::unpad_rows(&mapped, pending.width, pending.height, pending.bytes_per_row);
    if pending.swizzle_bgra {
        image_export::swizzle_bgra_to_rgba(&mut pixels);
    }
    if pending.crop {
        Ok(image_export::crop_to_content(
            &pixels,
            pending.width,
            pending.height,
            pending.background,
        ))
    } else {
        Ok((pending.width, pending.height, pixels))
    }
}

fn validate_export_size(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err(String::from("Image size must be greater than zero"));
    }
    if width > 8192 || height > 8192 {
        return Err(String::from("Image size exceeds the 8192px GPU limit"));
    }
    Ok(())
}

fn create_readback_buffer(device: &wgpu::Device, size: u64, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn copy_texture_to_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    staging: &wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Export Copy Encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));
}

async fn map_buffer(
    staging: &wgpu::Buffer,
    device: &wgpu::Device,
    range: std::ops::Range<u64>,
) -> Result<Vec<u8>, String> {
    let slice = staging.slice(range);
    let (sender, receiver) = flume::bounded(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    poll_map(device);
    match receiver.recv_async().await {
        Ok(Ok(())) => {
            let data = slice.get_mapped_range().to_vec();
            staging.unmap();
            Ok(data)
        }
        _ => {
            staging.unmap();
            Err(String::from("Failed to read GPU buffer"))
        }
    }
}

fn poll_map(device: &wgpu::Device) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = device;
    }
}
