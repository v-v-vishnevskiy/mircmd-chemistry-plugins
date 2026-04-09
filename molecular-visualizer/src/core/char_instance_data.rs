use super::super::types::Color;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CharInstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub uv_rect: [f32; 4],
    pub width: f32,
    pub char_x_offset: f32,
    pub depth_offset: f32,
    pub color: Color,
}

impl CharInstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CharInstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model matrix row 0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix row 1
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix row 2
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix row 3
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // UV Rect
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Width
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 5) as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32,
                },
                // Char X Offset
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 5 + 4) as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32,
                },
                // Depth Offset
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 5 + 8) as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 5 + 12) as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
