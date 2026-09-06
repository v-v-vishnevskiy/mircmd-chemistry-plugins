use super::isosurface::Isosurface;
use super::state::{Rgba, Surface, SurfaceGroup};
use super::types::Color;
use shared_lib::types::VolumeCube as VolumeCubeData;

pub struct VolumeCube {
    data: VolumeCubeData,
    groups: Vec<IsosurfaceGroup>,
    next_id: u32,
}

struct IsosurfaceGroup {
    id: u32,
    value: f64,
    visible: bool,
    surfaces: Vec<Isosurface>,
}

impl VolumeCube {
    pub fn new(data: VolumeCubeData) -> Self {
        Self {
            data,
            groups: Vec::new(),
            next_id: 1,
        }
    }

    pub fn is_empty_scalar_field(&self) -> bool {
        self.data.cube_data.is_empty()
            || self.data.steps_number.0 == 0
            || self.data.steps_number.1 == 0
            || self.data.steps_number.2 == 0
    }

    pub fn add_isosurface(
        &mut self,
        device: &wgpu::Device,
        value: f64,
        color_1: Color,
        color_2: Color,
        inverse: bool,
    ) -> Result<bool, String> {
        if self.is_empty_scalar_field() {
            return Ok(false);
        }
        let group_id = self.next_id;
        self.next_id = self.next_id.saturating_add(3);
        let surfaces = self.build_surfaces(device, group_id, value, color_1, color_2, inverse)?;
        self.groups.push(IsosurfaceGroup {
            id: group_id,
            value: value.abs(),
            visible: true,
            surfaces,
        });
        Ok(true)
    }

    fn build_surfaces(
        &self,
        device: &wgpu::Device,
        group_id: u32,
        value: f64,
        color_1: Color,
        color_2: Color,
        inverse: bool,
    ) -> Result<Vec<Isosurface>, String> {
        let abs_value = value.abs();
        let first = Isosurface::try_new(
            device,
            &self.data,
            group_id + 1,
            value < 0.0,
            color_1,
            abs_value,
            if value < 0.0 { -1.0 } else { 1.0 },
        )?;
        let mut surfaces = vec![first];
        if inverse && value != 0.0 {
            if let Ok(inverted) = Isosurface::try_new(
                device,
                &self.data,
                group_id + 2,
                value > 0.0,
                color_2,
                abs_value,
                if value > 0.0 { -1.0 } else { 1.0 },
            ) {
                surfaces.push(inverted);
            }
        }
        Ok(surfaces)
    }

    pub fn set_isosurface_color(&mut self, queue: &wgpu::Queue, id: u32, color: Color) {
        if let Some(surface) = self.find_surface_mut(id) {
            surface.set_color(queue, color);
        }
    }

    pub fn set_isosurface_visible(&mut self, id: u32, visible: bool, apply_to_children: bool, apply_to_parents: bool) {
        if let Some(group) = self.groups.iter_mut().find(|group| group.id == id) {
            group.visible = visible;
            if apply_to_children {
                for surface in &mut group.surfaces {
                    surface.visible = visible;
                }
            }
            return;
        }
        self.set_child_visible(id, visible, apply_to_parents);
    }

    pub fn remove_isosurface(&mut self, id: u32) {
        if let Some(index) = self.groups.iter().position(|group| group.id == id) {
            self.groups.remove(index);
            return;
        }
        for group in &mut self.groups {
            group.surfaces.retain(|surface| surface.id != id);
        }
        self.groups.retain(|group| !group.surfaces.is_empty());
    }

    pub fn surface_groups(&self) -> Vec<SurfaceGroup> {
        self.groups
            .iter()
            .map(|group| {
                let surfaces: Vec<Surface> = group
                    .surfaces
                    .iter()
                    .map(|surface| {
                        Surface::new(
                            surface.id,
                            surface.inverted,
                            surface.visible,
                            color_to_rgba(surface.color),
                        )
                    })
                    .collect();
                SurfaceGroup::new(group.id, group.value, group.visible, surfaces)
            })
            .collect()
    }

    fn render_isosurface(&self, render_pass: &mut wgpu::RenderPass, opaque: bool) {
        for group in &self.groups {
            if !group.visible {
                continue;
            }
            for isosurface in &group.surfaces {
                let is_opaque = isosurface.color.a >= 1.0;
                if isosurface.visible && is_opaque == opaque {
                    isosurface.render(render_pass);
                }
            }
        }
    }

    pub fn render_opaque(&self, render_pass: &mut wgpu::RenderPass) {
        self.render_isosurface(render_pass, true);
    }

    pub fn render_transparent(&self, render_pass: &mut wgpu::RenderPass) {
        self.render_isosurface(render_pass, false);
    }

    fn find_surface_mut(&mut self, id: u32) -> Option<&mut Isosurface> {
        for group in &mut self.groups {
            if let Some(surface) = group.surfaces.iter_mut().find(|surface| surface.id == id) {
                return Some(surface);
            }
        }
        None
    }

    fn set_child_visible(&mut self, id: u32, visible: bool, apply_to_parents: bool) {
        for group in &mut self.groups {
            if let Some(surface) = group.surfaces.iter_mut().find(|surface| surface.id == id) {
                surface.visible = visible;
                if apply_to_parents && visible {
                    group.visible = true;
                }
                return;
            }
        }
    }
}

fn color_to_rgba(color: Color) -> Rgba {
    Rgba::new(color.r, color.g, color.b, color.a)
}
