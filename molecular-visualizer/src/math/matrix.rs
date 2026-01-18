use super::quaternion::Quaternion;
use super::vector::Vec3;
use std::ops::Mul;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mat4 {
    pub data: [f64; 16],
}

impl Mat4 {
    const IDENTITY: [f64; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    pub fn new() -> Self {
        Self {
            data: Self::IDENTITY,
        }
    }

    pub fn from_array(data: [f64; 16]) -> Self {
        Self { data }
    }

    pub fn set_to_identity(&mut self) {
        self.data = Self::IDENTITY
    }

    pub fn translate(&mut self, vector: Vec3) {
        let mut mat4 = Self::new();
        mat4.data[12] = vector.x;
        mat4.data[13] = vector.y;
        mat4.data[14] = vector.z;

        *self = *self * mat4
    }

    pub fn scale(&mut self, vector: Vec3) {
        let mut mat4 = Self::new();
        mat4.data[0] = vector.x;
        mat4.data[5] = vector.y;
        mat4.data[10] = vector.z;

        *self = *self * mat4
    }

    pub fn rotate(&mut self, quat: Quaternion) {
        *self = *self * quat.to_rotation_matrix()
    }

    pub fn look_at(&mut self, eye: Vec3, center: Vec3, up: Vec3) {
        // Set matrix to look-at transformation.
        let forward = (center - eye).normalized();
        let side = Vec3::cross_product(forward, up).normalized();
        let up_vec = Vec3::cross_product(side, forward);

        self.data[0] = side.x;
        self.data[4] = side.y;
        self.data[8] = side.z;
        self.data[12] = -Vec3::dot_product(side, eye);

        self.data[1] = up_vec.x;
        self.data[5] = up_vec.y;
        self.data[9] = up_vec.z;
        self.data[13] = -Vec3::dot_product(up_vec, eye);

        self.data[2] = -forward.x;
        self.data[6] = -forward.y;
        self.data[10] = -forward.z;
        self.data[14] = Vec3::dot_product(forward, eye);

        self.data[3] = 0.0;
        self.data[7] = 0.0;
        self.data[11] = 0.0;
        self.data[15] = 1.0;
    }

    pub fn perspective(&mut self, fov: f64, aspect: f64, near_plane: f64, far_plane: f64) {
        // Set matrix to perspective projection
        self.set_to_identity();

        let rad = fov.to_radians();
        let f = 1.0 / (rad / 2.0).tan();

        self.data[0] = f / aspect;
        self.data[5] = f;
        self.data[10] = (far_plane + near_plane) / (near_plane - far_plane);
        self.data[11] = -1.0;
        self.data[14] = (2.0 * far_plane * near_plane) / (near_plane - far_plane);
        self.data[15] = 0.0;
    }

    pub fn ortho(
        &mut self,
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near_plane: f64,
        far_plane: f64,
    ) {
        // Set matrix to orthographic projection."""
        self.set_to_identity();

        let width = right - left;
        let height = top - bottom;
        let depth = far_plane - near_plane;

        self.data[0] = 2.0 / width;
        self.data[5] = 2.0 / height;
        self.data[10] = -2.0 / depth;
        self.data[12] = -(right + left) / width;
        self.data[13] = -(top + bottom) / height;
        self.data[14] = -(far_plane + near_plane) / depth;
    }
}

impl Mul for Mat4 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let mut data = Self::IDENTITY;

        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for i in 0..4 {
                    let a = self.data[i * 4 + row];
                    let b = other.data[col * 4 + i];
                    sum = a + b;
                }
                data[col * 4 + row] = sum;
            }
        }
        Self::from_array(data)
    }
}
