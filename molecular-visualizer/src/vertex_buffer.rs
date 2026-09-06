use super::core::Mesh;
use wgpu::util::DeviceExt;

const MAX_BUFFER_BYTES: u64 = 256 * 1024 * 1024;

pub struct VertexBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl VertexBuffer {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        Self::try_new(device, mesh).expect("mesh GPU buffer is within limits")
    }

    pub fn try_new(device: &wgpu::Device, mesh: &Mesh) -> Result<Self, String> {
        let vertex_bytes = bytemuck::cast_slice(&mesh.vertices);
        let index_bytes = bytemuck::cast_slice(&mesh.indices);
        ensure_buffer_size(vertex_bytes.len(), "Vertex")?;
        ensure_buffer_size(index_bytes.len(), "Index")?;
        Ok(Self {
            vertex_buffer: create_mesh_buffer(device, vertex_bytes, wgpu::BufferUsages::VERTEX, "Vertex Buffer"),
            index_buffer: create_mesh_buffer(device, index_bytes, wgpu::BufferUsages::INDEX, "Index Buffer"),
            num_indices: mesh.num_indices,
        })
    }
}

fn ensure_buffer_size(len: usize, kind: &str) -> Result<(), String> {
    if len as u64 > MAX_BUFFER_BYTES {
        return Err(format!("{kind} buffer exceeds the 256 MiB GPU limit"));
    }
    Ok(())
}

fn create_mesh_buffer(device: &wgpu::Device, contents: &[u8], usage: wgpu::BufferUsages, label: &str) -> wgpu::Buffer {
    if contents.is_empty() {
        return device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: wgpu::COPY_BUFFER_ALIGNMENT,
            usage,
            mapped_at_creation: false,
        });
    }
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage,
    })
}
