use super::core::mesh::InstanceData;
use super::core::{Mat4, Quaternion, Vec3};
use super::types::Color;
use super::utils::get_model_matrix;

pub struct Bond {
    pub position: Vec3<f32>,
    pub direction: Vec3<f32>,
    pub thickness: f32,
    pub lenght: f32,
    pub color: Color,
    pub visible: bool,
}

impl Bond {
    pub fn new(position: Vec3<f32>, direction: Vec3<f32>, thickness: f32, lenght: f32, color: Color) -> Self {
        Self {
            position,
            direction,
            thickness,
            lenght,
            color,
            visible: true,
        }
    }

    pub fn get_instance_data(&self) -> InstanceData {
        let rotation = Quaternion::rotation_to(Vec3::new(0.0, 0.0, 1.0), self.direction);
        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(self.position);
        transform.rotate(rotation);
        transform.scale(Vec3::new(self.thickness, self.thickness, self.lenght));

        InstanceData {
            model_matrix: get_model_matrix(&transform),
            color: self.color,
            picking_color: Color::new(0.0, 0.0, 0.0, 1.0),
            ray_casting_type: 2,
        }
    }
}
