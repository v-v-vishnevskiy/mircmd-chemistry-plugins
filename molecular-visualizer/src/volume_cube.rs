use super::isosurface::Isosurface;
use super::types::Color;
use shared_lib::types::VolumeCube as VolumeCubeData;

pub struct VolumeCube {
    data: VolumeCubeData,

    isosurfaces: Vec<Isosurface>,
}

impl VolumeCube {
    pub fn new(data: VolumeCubeData) -> Self {
        Self {
            data,
            isosurfaces: Vec::new(),
        }
    }

    pub fn add_isosurface(&mut self, device: &wgpu::Device, color: Color, value: f64, factor: f64) {
        self.isosurfaces
            .push(Isosurface::new(device, &self.data, color, value, factor));
    }

    fn render_isosurface(&self, render_pass: &mut wgpu::RenderPass, opaque: bool) {
        for isosurface in &self.isosurfaces {
            if isosurface.visible
                && (opaque == true && isosurface.color.a == 1.0 || opaque == false && isosurface.color.a < 1.0)
            {
                isosurface.render(render_pass);
            }
        }
    }

    pub fn render_opaque(&self, render_pass: &mut wgpu::RenderPass) {
        self.render_isosurface(render_pass, true);
    }

    pub fn render_transparent(&self, render_pass: &mut wgpu::RenderPass) {
        self.render_isosurface(render_pass, false);
    }
}
