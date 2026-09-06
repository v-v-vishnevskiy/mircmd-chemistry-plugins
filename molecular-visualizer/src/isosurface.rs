use super::core::instance_data::InstanceData;
use super::core::mesh_objects::marching_cubes::isosurface;
use super::core::{Mat4, Vec3};
use super::types::Color;
use super::utils::get_model_matrix;
use super::vertex_buffer::VertexBuffer;
use shared_lib::types::VolumeCube as VolumeCubeData;
use wgpu::util::DeviceExt;

const BOHR2ANGSTROM: f32 = 0.529177210903;
const COLOR_BYTE_OFFSET: u64 = 64;

pub struct Isosurface {
    pub id: u32,
    pub inverted: bool,
    pub color: Color,
    pub visible: bool,
    instance_data: InstanceData,
    isosurface_vb: VertexBuffer,
    isosurface_instance_buffer: wgpu::Buffer,
}

impl Isosurface {
    pub fn try_new(
        device: &wgpu::Device,
        data: &VolumeCubeData,
        id: u32,
        inverted: bool,
        color: Color,
        value: f64,
        factor: f64,
    ) -> Result<Self, String> {
        let mesh = isosurface(&data.cube_data, data.steps_number, value, factor);
        let isosurface_vb = VertexBuffer::try_new(device, &mesh)?;
        Ok(Self::from_mesh(device, data, id, inverted, color, isosurface_vb))
    }

    fn from_mesh(
        device: &wgpu::Device,
        data: &VolumeCubeData,
        id: u32,
        inverted: bool,
        color: Color,
        isosurface_vb: VertexBuffer,
    ) -> Self {
        let transform = cube_transform(data);
        let instance_data = InstanceData {
            model_matrix: get_model_matrix(&transform),
            color,
            picking_color: Color::new(0.0, 0.0, 0.0, 1.0),
            lighting_model: 1,
            ray_casting_type: 0,
        };
        let isosurface_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Isosurface Instance Buffer"),
            contents: bytemuck::cast_slice(&[instance_data]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            id,
            inverted,
            color,
            visible: true,
            instance_data,
            isosurface_vb,
            isosurface_instance_buffer,
        }
    }

    pub fn set_color(&mut self, queue: &wgpu::Queue, color: Color) {
        self.color = color;
        self.instance_data.color = color;
        queue.write_buffer(
            &self.isosurface_instance_buffer,
            COLOR_BYTE_OFFSET,
            bytemuck::bytes_of(&color),
        );
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if self.isosurface_vb.num_indices == 0 {
            return;
        }
        render_pass.set_vertex_buffer(0, self.isosurface_vb.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.isosurface_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(1, self.isosurface_instance_buffer.slice(..));
        render_pass.draw_indexed(0..self.isosurface_vb.num_indices, 0, 0..1);
    }
}

fn cube_transform(data: &VolumeCubeData) -> Mat4<f32> {
    let mut transform = Mat4::new();
    transform.translate(
        Vec3::new(
            data.box_origin[0] as f32,
            data.box_origin[1] as f32,
            data.box_origin[2] as f32,
        ) * BOHR2ANGSTROM,
    );
    transform.scale(
        Vec3::new(
            data.steps_size[0][0] as f32,
            data.steps_size[1][1] as f32,
            data.steps_size[2][2] as f32,
        ) * BOHR2ANGSTROM,
    );
    transform
}
