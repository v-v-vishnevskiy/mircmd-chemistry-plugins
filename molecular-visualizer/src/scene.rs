use super::config::Config;
use super::core::{Camera, Mesh, ProjectionManager, ProjectionMode, Transform, Vec3, mesh_objects};
use super::molecule::Molecule;
use super::renderer::Renderer;
use super::vertex_buffer::VertexBuffer;
use shared_lib::types::AtomicCoordinates;

pub struct Scene {
    pub projection_manager: ProjectionManager,
    pub transform: Transform,
    pub renderer: Renderer,

    camera: Camera,
    molecule: Option<Molecule>,
    cube_mesh: Mesh,
    cube_vb: VertexBuffer,
}

impl Scene {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let cube_mesh = mesh_objects::cube::create(2.0);
        Self {
            renderer: Renderer::new(device, config),
            projection_manager: ProjectionManager::new(1, 1, ProjectionMode::Perspective),
            camera: Camera::new(),
            molecule: None,
            transform: Transform::new(),
            cube_vb: VertexBuffer::new(device, &cube_mesh),
            cube_mesh,
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

    pub fn render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue, config: &Config) {
        let molecule = match &self.molecule {
            Some(molecule) => molecule,
            None => return,
        };

        // Calculate matrices
        let projection_matrix = *self.projection_manager.get_matrix();
        let view_matrix = *self.camera.get_matrix();
        let scene_matrix = *self.transform.get_matrix() * molecule.transform;
        let final_matrix = projection_matrix * view_matrix * scene_matrix;
        let render_mode = 0u32;
        let is_perspective = self.projection_manager.mode == ProjectionMode::Perspective;
        let lighting_model = 1u32;

        // Update uniform buffer with all 4 matrices + projection type flag
        // matrix = (16 float × 4 байта) = 64 bytes
        let mut uniforms_data = [0u8; 272];
        uniforms_data[0..64].copy_from_slice(bytemuck::cast_slice(&projection_matrix.data));
        uniforms_data[64..128].copy_from_slice(bytemuck::cast_slice(&view_matrix.data));
        uniforms_data[128..192].copy_from_slice(bytemuck::cast_slice(&scene_matrix.data));
        uniforms_data[192..256].copy_from_slice(bytemuck::cast_slice(&final_matrix.data));
        uniforms_data[256..260].copy_from_slice(&render_mode.to_le_bytes());
        uniforms_data[260..264].copy_from_slice(&(if is_perspective { 1u32 } else { 0u32 }).to_le_bytes());
        uniforms_data[264..268].copy_from_slice(&lighting_model.to_le_bytes());

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

        // Begin render pass
        {
            let bg_color = if render_mode == 1 {
                wgpu::Color {
                    // picking
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                wgpu::Color {
                    r: config.style.background_color.r as f64,
                    g: config.style.background_color.g as f64,
                    b: config.style.background_color.b as f64,
                    a: 1.0,
                }
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg_color),
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

            // Render atoms
            render_pass.set_vertex_buffer(1, molecule.atoms_instance_buffer.slice(..));
            render_pass.draw_indexed(0..self.cube_mesh.num_indices, 0, 0..molecule.atoms_instance_count());

            // Render bonds
            if render_mode == 0 && molecule.bonds_instance_count() > 0 {
                render_pass.set_vertex_buffer(1, molecule.bonds_instance_buffer.slice(..));
                render_pass.draw_indexed(0..self.cube_mesh.num_indices, 0, 0..molecule.bonds_instance_count());
            }
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
