use super::super::types::Color;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub color: Color,
    pub picking_color: Color,
    pub lighting_model: u32,
    pub ray_casting_type: u32,
}

impl InstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model matrix row 0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix row 1
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix row 2
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix row 3
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress, // offset 64
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Picking color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 5) as wgpu::BufferAddress, // offset 80
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Lighting model
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 6) as wgpu::BufferAddress, // offset 96
                    shader_location: 8,
                    format: wgpu::VertexFormat::Uint32,
                },
                // Ray Casting Type
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 6 + std::mem::size_of::<u32>()) as wgpu::BufferAddress, // offset 100
                    shader_location: 9,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub num_indices: u32,
}
