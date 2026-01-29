use super::config::Config;
use super::core::{Camera, Mesh, ProjectionManager, ProjectionMode, Transform, Vec3, mesh_objects};
use super::molecule::Molecule;
use super::renderer::Renderer;
use super::vertex_buffer_object::VertexBufferObject;
use shared_lib::types::AtomicCoordinates;

pub struct Scene {
    pub projection_manager: ProjectionManager,
    pub transform: Transform,
    pub renderer: Renderer,

    camera: Camera,
    molecule: Option<Molecule>,
    cube_mesh: Mesh,
    cube_vbo: VertexBufferObject,
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
            cube_vbo: VertexBufferObject::new(device, &cube_mesh),
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
            .set_near_far_plane(scene_size / fov_factor, 8.0 * scene_size / fov_factor);

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

        // Calculate view-projection matrix
        let camera_matrix = *self.camera.get_matrix();
        let scene_matrix = *self.transform.get_matrix();
        let view_projection = *self.projection_manager.get_matrix() * camera_matrix * scene_matrix * molecule.transform;

        // Update uniform buffer
        queue.write_buffer(
            &self.renderer.uniform_buffer,
            0,
            bytemuck::cast_slice(&view_projection.data),
        );

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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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
            render_pass.set_vertex_buffer(0, self.cube_vbo.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.cube_vbo.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);

            // Render atoms
            render_pass.set_vertex_buffer(1, molecule.atoms_instance_buffer.slice(..));
            render_pass.draw_indexed(0..self.cube_mesh.num_indices, 0, 0..molecule.atoms_instance_count());

            // Render bonds
            if molecule.bonds_instance_count() > 0 {
                render_pass.set_vertex_buffer(1, molecule.bonds_instance_buffer.slice(..));
                render_pass.draw_indexed(0..self.cube_mesh.num_indices, 0, 0..molecule.bonds_instance_count());
            }
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
