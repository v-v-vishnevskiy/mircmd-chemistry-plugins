use wasm_bindgen::prelude::*;

use super::core::{CharInstanceData, InstanceData, Mat4, Vec3};
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
    pub atomic_number: i32,
    pub position: Vec3<f32>,
    pub radius: f32,
    pub color: Color,
    pub picking_color: Color,
    pub bounding_sphere_color: Color,
    pub bounding_sphere_scale_factor: f32,
    pub visible: bool,
    pub highlighted: bool,
    pub selected: bool,
    pub symbol_visible: bool,
    pub number_visible: bool,
}

impl Atom {
    pub fn new(
        number: i32,
        atomic_number: i32,
        position: Vec3<f32>,
        radius: f32,
        color: Color,
        picking_color: Color,
        bounding_sphere_color: Color,
        bounding_sphere_scale_factor: f32,
        symbol_visible: bool,
        number_visible: bool,
    ) -> Self {
        Self {
            number,
            atomic_number,
            position,
            radius,
            color,
            picking_color,
            bounding_sphere_color,
            bounding_sphere_scale_factor,
            visible: true,
            highlighted: false,
            selected: false,
            symbol_visible,
            number_visible,
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
        }
    }

    pub fn get_label_instance_data(&self, color: Color, size: f32, offset: f32, font_atlas: &super::core::FontAtlas) -> Vec<(char, CharInstanceData)> {
        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(self.position);
        transform.scale(Vec3::new(size, size, size));

        let mut data = Vec::new();

        let mut text = String::new();
        if self.symbol_visible {
            if let Some(element) = shared_lib::periodic_table::get_element_by_number(self.atomic_number) {
                text.push_str(&element.symbol);
            }
        }
        if self.number_visible {
            text.push_str(&self.number.to_string());
        }

        let gap = 0.2;
        let mut total_width = 0.0;
        let mut chars_info = Vec::with_capacity(text.len());
        
        for c in text.chars() {
            let info = *font_atlas.get_char_info(c);
            let char_width = (info.width / info.height) * 2.0;
            chars_info.push((c, info, char_width));
            total_width += char_width + gap;
        }
        
        if total_width > 0.0 {
            total_width -= gap;
        }

        let mut current_left = -total_width / 2.0;

        for (c, info, char_width) in chars_info {
            data.push((
                c,
                CharInstanceData {
                    model_matrix: get_model_matrix(&transform),
                    uv_rect: [info.u_min, info.v_min, info.u_max, info.v_max],
                    width: info.width / info.height,
                    char_x_offset: current_left + char_width / 2.0,
                    depth_offset: self.radius + offset,
                    color,
                },
            ));
            current_left += char_width + gap;
        }

        data
    }
}
