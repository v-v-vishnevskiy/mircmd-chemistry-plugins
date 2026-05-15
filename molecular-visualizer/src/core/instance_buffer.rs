pub struct InstanceBuffer {
    pub buffer: wgpu::Buffer,
    pub count: usize,
}

impl InstanceBuffer {
    pub fn new(buffer: wgpu::Buffer, count: usize) -> Self {
        Self { buffer, count }
    }
}
