use super::matrix::Mat4;
use super::vector::Vec3;
use std::ops::Mul;

const EPSILON: f64 = 1e-10;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    fn length(&self) -> f64 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalized(&self) -> Self {
        let len = self.length();
        if len < EPSILON {
            Self::new(1.0, 0.0, 0.0, 0.0)
        } else {
            Self::new(self.w / len, self.x / len, self.y / len, self.z / len)
        }
    }

    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    pub fn from_axis_and_angle(axis: Vec3, angle: f64) -> Self {
        let rad = angle.to_radians();
        let half_angle = rad / 2.0;
        let sin = half_angle.sin();
        let axis_normalized = axis.normalized();

        Self {
            w: half_angle.cos(),
            x: axis_normalized.x * sin,
            y: axis_normalized.y * sin,
            z: axis_normalized.z * sin,
        }
    }

    pub fn rotation_to(from_vec: Vec3, to_vec: Vec3) -> Self {
        // Create quaternion that rotates from one vector to another.

        let v1 = from_vec.normalized();
        let v2 = to_vec.normalized();

        let dot = Vec3::dot_product(v1, v2);

        // Vectors are parallel
        if dot >= 0.9999 {
            return Self::new(1.0, 0.0, 0.0, 0.0);
        }

        // Vectors are opposite
        if dot <= -0.9999 {
            // Find orthogonal vector
            let mut axis = Vec3::cross_product(Vec3::new(1.0, 0.0, 0.0), v1);
            if axis.length() < 0.0001 {
                axis = Vec3::cross_product(Vec3::new(0.0, 1.0, 0.0), v1);
            }
            axis = axis.normalized();
            return Self::new(0.0, axis.x, axis.y, axis.z);
        }

        // General case
        let axis = Vec3::cross_product(v1, v2);
        let w = (v1.length_squared() * v2.length_squared()).sqrt() + dot;

        Self::new(w, axis.x, axis.y, axis.z).normalized()
    }

    pub fn to_rotation_matrix(self) -> Mat4 {
        let q = self.normalized();

        Mat4::from_array([
            // Column 0
            1.0 - 2.0 * (q.y * q.y + q.z * q.z),
            2.0 * (q.x * q.y + q.w * q.z),
            2.0 * (q.x * q.z - q.w * q.y),
            0.0,
            // Column 1
            2.0 * (q.x * q.y - q.w * q.z),
            1.0 - 2.0 * (q.x * q.x + q.z * q.z),
            2.0 * (q.y * q.z + q.w * q.x),
            0.0,
            // Column 2
            2.0 * (q.x * q.z + q.w * q.y),
            2.0 * (q.y * q.z - q.w * q.x),
            1.0 - 2.0 * (q.x * q.x + q.y * q.y),
            0.0,
            // Column 3
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    pub fn approx_eq(&self, other: Quaternion) -> bool {
        (self.w - other.w).abs() < EPSILON
            && (self.x - other.x).abs() < EPSILON
            && (self.y - other.y).abs() < EPSILON
            && (self.z - other.z).abs() < EPSILON
    }
}

impl Mul for Quaternion {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}
