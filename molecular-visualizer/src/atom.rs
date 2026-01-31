use super::core::mesh::InstanceData;
use super::core::{Mat4, Vec3};
use super::types::Color;
use super::utils::get_model_matrix;

pub struct Atom {
    pub position: Vec3<f32>,
    pub radius: f32,
    pub color: Color,
    pub picking_color: Color,
    pub visible: bool,
    pub highlighted: bool,
    pub selected: bool,
}

impl Atom {
    pub fn new(position: Vec3<f32>, radius: f32, color: Color, picking_color: Color) -> Self {
        Self {
            position,
            radius,
            color,
            picking_color,
            visible: true,
            highlighted: false,
            selected: false,
        }
    }

    pub fn get_instance_data(&self) -> InstanceData {
        let radius = if self.highlighted {
            self.radius * 1.15
        } else {
            self.radius
        };

        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(self.position);
        transform.scale(Vec3::new(radius, radius, radius));

        InstanceData {
            model_matrix: get_model_matrix(&transform),
            color: self.color,
            picking_color: self.picking_color,
            ray_casting_type: 1,
        }
    }
}
