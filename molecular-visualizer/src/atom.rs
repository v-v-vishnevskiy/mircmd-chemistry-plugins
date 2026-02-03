use wasm_bindgen::prelude::*;

use super::core::mesh::InstanceData;
use super::core::{Mat4, Vec3};
use super::types::Color;
use super::utils::get_model_matrix;

#[wasm_bindgen]
pub struct AtomInfo {
    symbol: String,
    tag: usize,
}

#[wasm_bindgen]
impl AtomInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(symbol: String, tag: usize) -> Self {
        Self { symbol, tag }
    }

    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn tag(&self) -> usize {
        self.tag
    }
}

pub struct Atom {
    pub number: i32,
    pub position: Vec3<f32>,
    pub radius: f32,
    pub color: Color,
    pub picking_color: Color,
    pub bounding_sphere_color: Color,
    pub bounding_sphere_scale_factor: f32,
    pub visible: bool,
    pub highlighted: bool,
    pub selected: bool,
}

impl Atom {
    pub fn new(
        number: i32,
        position: Vec3<f32>,
        radius: f32,
        color: Color,
        picking_color: Color,
        bounding_sphere_color: Color,
        bounding_sphere_scale_factor: f32,
    ) -> Self {
        Self {
            number,
            position,
            radius,
            color,
            picking_color,
            bounding_sphere_color,
            bounding_sphere_scale_factor,
            visible: true,
            highlighted: false,
            selected: false,
        }
    }

    pub fn toggle_selection(&mut self) {
        self.selected = !self.selected;
    }

    pub fn get_instance_data(&self, bounding_sphere: bool) -> InstanceData {
        let radius_factor = if bounding_sphere {
            self.bounding_sphere_scale_factor
        } else {
            1.0
        };

        let radius = if self.highlighted {
            self.radius * 1.15 * radius_factor
        } else {
            self.radius * radius_factor
        };

        let color = if bounding_sphere {
            self.bounding_sphere_color
        } else {
            self.color
        };

        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(self.position);
        transform.scale(Vec3::new(radius, radius, radius));

        InstanceData {
            model_matrix: get_model_matrix(&transform),
            color: color,
            picking_color: self.picking_color,
            lighting_model: if bounding_sphere { 0 } else { 1 },
            ray_casting_type: 1,
            use_texture: 0,
        }
    }
}
