use shared_lib::types::AtomicCoordinates;

use super::atom::AtomInfo;
use super::config::Config;
use super::core::{Camera, Mesh, ProjectionManager, ProjectionMode, Transform, Vec3, mesh_objects};
use super::molecule::Molecule;
use super::renderer::Renderer;
use super::utils::color_to_id;
use super::vertex_buffer::VertexBuffer;

pub struct Scene {
    pub projection_manager: ProjectionManager,
    pub transform: Transform,
    pub renderer: Renderer,

    camera: Camera,
    molecule: Option<Molecule>,
    cube_mesh: Mesh,
    cube_vb: VertexBuffer,

    picking_texture_dirty: bool,
}

impl Scene {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let cube_mesh = mesh_objects::cube::create(2.0);
        Self {
            projection_manager: ProjectionManager::new(1, 1, ProjectionMode::Perspective),
            transform: Transform::new(),
            renderer: Renderer::new(device, surface_config),
            camera: Camera::new(),
            molecule: None,
            cube_vb: VertexBuffer::new(device, &cube_mesh),
            cube_mesh,
            picking_texture_dirty: true,
        }
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

    pub fn load_atomic_coordinates(&mut self, device: &wgpu::Device, config: &Config, data: &AtomicCoordinates) {
        match Molecule::new(device, config, data) {
            Ok(molecule) => {
                self.setup_camera(molecule.radius);
                self.molecule = Some(molecule);
            }
            Err(_) => {}
        }
    }

    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &Config,
        render_mode: u32,
    ) {
        let molecule = match &self.molecule {
            Some(molecule) => molecule,
            None => return,
        };

        // Calculate matrices
        let projection_matrix = *self.projection_manager.get_matrix();
        let view_matrix = *self.camera.get_matrix();
        let scene_matrix = *self.transform.get_matrix() * molecule.transform;
        let final_matrix = projection_matrix * view_matrix * scene_matrix;
        let is_perspective = self.projection_manager.mode == ProjectionMode::Perspective;

        // Update uniform buffer with all 4 matrices + projection type flag
        // matrix = (16 float × 4 байта) = 64 bytes
        let mut uniforms_data = [0u8; 272];
        uniforms_data[0..64].copy_from_slice(bytemuck::cast_slice(&projection_matrix.data));
        uniforms_data[64..128].copy_from_slice(bytemuck::cast_slice(&view_matrix.data));
        uniforms_data[128..192].copy_from_slice(bytemuck::cast_slice(&scene_matrix.data));
        uniforms_data[192..256].copy_from_slice(bytemuck::cast_slice(&final_matrix.data));
        uniforms_data[256..260].copy_from_slice(&render_mode.to_le_bytes());
        uniforms_data[260..264].copy_from_slice(&(if is_perspective { 1u32 } else { 0u32 }).to_le_bytes());

        queue.write_buffer(&self.renderer.uniform_buffer, 0, &uniforms_data);

        // Get current texture from surface
        let surface_texture = match surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(_) => return,
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let has_transparent_objects = molecule.bounding_spheres_instance_count() > 0;

        // Pass 1: Render opaque objects
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Opaque Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: config.style.background_color.r as f64,
                            g: config.style.background_color.g as f64,
                            b: config.style.background_color.b as f64,
                            a: 1.0,
                        }),
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
            render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);

            // Render atoms (opaque)
            if molecule.atoms_instance_count() > 0 {
                render_pass.set_vertex_buffer(1, molecule.atoms_instance_buffer.slice(..));
                render_pass.draw_indexed(
                    0..self.cube_mesh.num_indices,
                    0,
                    0..molecule.atoms_instance_count() as u32,
                );
            }

            // Render bonds (opaque)
            if molecule.bonds_instance_count() > 0 {
                render_pass.set_vertex_buffer(1, molecule.bonds_instance_buffer.slice(..));
                render_pass.draw_indexed(
                    0..self.cube_mesh.num_indices,
                    0,
                    0..molecule.bonds_instance_count() as u32,
                );
            }
        }

        // Pass 2 & 3: WBOIT for transparent objects
        if has_transparent_objects {
            // Pass 2: Render transparent objects to WBOIT buffers
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("WBOIT Transparent Pass"),
                    color_attachments: &[
                        // Accumulation texture
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.renderer.wboit_accumulation_texture_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                        // Revealage texture
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
                            load: wgpu::LoadOp::Load, // Keep depth from opaque pass
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                render_pass.set_pipeline(&self.renderer.transparent_pipeline);
                render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);

                // Render bounding spheres (transparent)
                render_pass.set_vertex_buffer(1, molecule.atom_selections_instance_buffer.slice(..));
                render_pass.draw_indexed(
                    0..self.cube_mesh.num_indices,
                    0,
                    0..molecule.bounding_spheres_instance_count() as u32,
                );
            }

            // Pass 3: Composite WBOIT result onto framebuffer
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("WBOIT Composite Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // Keep opaque rendering
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
                render_pass.draw(0..6, 0..1); // Full-screen quad
            }
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        self.picking_texture_dirty = true;
    }

    fn render_picking_pass(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let molecule = match &self.molecule {
            Some(molecule) => molecule,
            None => return,
        };

        // Calculate matrices (same as main render)
        let projection_matrix = *self.projection_manager.get_matrix();
        let view_matrix = *self.camera.get_matrix();
        let scene_matrix = *self.transform.get_matrix() * molecule.transform;
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
            render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);

            // Render atoms only (bonds don't have picking IDs)
            render_pass.set_vertex_buffer(1, molecule.atoms_instance_buffer.slice(..));
            render_pass.draw_indexed(
                0..self.cube_mesh.num_indices,
                0,
                0..molecule.atoms_instance_count() as u32,
            );
        }

        queue.submit(std::iter::once(encoder.finish()));
        self.picking_texture_dirty = false;
    }

    pub async fn read_picking_pixel(&self, x: u32, y: u32, device: &wgpu::Device, queue: &wgpu::Queue) -> usize {
        let (width, height) = self.renderer.get_size();
        if x >= width || y >= height {
            return 0;
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Picking Read Encoder"),
        });

        // Copy single pixel from picking texture to staging buffer
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.renderer.picking_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.renderer.picking_staging_buffer,
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

        // Map buffer asynchronously
        let buffer_slice = self.renderer.picking_staging_buffer.slice(..4);

        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        match receiver.recv_async().await {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                let pixel = [data[0], data[1], data[2], data[3]];
                drop(data);
                self.renderer.picking_staging_buffer.unmap();

                color_to_id(pixel[0], pixel[1], pixel[2])
            }
            _ => {
                self.renderer.picking_staging_buffer.unmap();
                0
            }
        }
    }

    /// Returns (atom_info, needs_render)
    pub async fn new_cursor_position(
        &mut self,
        x: u32,
        y: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (Option<AtomInfo>, bool) {
        if self.molecule.is_none() {
            return (None, false);
        }

        if self.picking_texture_dirty {
            self.render_picking_pass(device, queue);
        }

        let atom_index = self.read_picking_pixel(x, y, device, queue).await;

        let molecule = self.molecule.as_mut().unwrap();
        molecule.highlight_atom(atom_index, device)
    }

    pub async fn toggle_atom_selection(&mut self, x: u32, y: u32, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.molecule.is_none() {
            return false;
        }

        if self.picking_texture_dirty {
            self.render_picking_pass(device, queue);
        }

        let atom_index = self.read_picking_pixel(x, y, device, queue).await;

        let molecule = self.molecule.as_mut().unwrap();
        molecule.toggle_atom_selection(atom_index, device)
    }
}
