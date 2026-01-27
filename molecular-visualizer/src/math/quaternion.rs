use super::matrix::Mat4;
use super::vector::Vec3;
use num_traits::Float;
use std::ops::Mul;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Quaternion<T: Float> {
    pub w: T,
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Float> Quaternion<T> {
    fn epsilon() -> T {
        T::from(1e-10).unwrap()
    }

    fn length(&self) -> T {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalized(&self) -> Self {
        let len = self.length();
        if len < Self::epsilon() {
            Self::new(T::one(), T::zero(), T::zero(), T::zero())
        } else {
            Self::new(self.w / len, self.x / len, self.y / len, self.z / len)
        }
    }

    pub fn new(w: T, x: T, y: T, z: T) -> Self {
        Self { w, x, y, z }
    }

    pub fn from_axis_and_angle(axis: Vec3<T>, angle: T) -> Self {
        let rad = angle.to_radians();
        let half_angle = rad / (T::one() + T::one());
        let sin = half_angle.sin();
        let axis_normalized = axis.normalized();

        Self {
            w: half_angle.cos(),
            x: axis_normalized.x * sin,
            y: axis_normalized.y * sin,
            z: axis_normalized.z * sin,
        }
    }

    pub fn rotation_to(from_vec: Vec3<T>, to_vec: Vec3<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        let threshold = T::from(0.9999).unwrap();
        let small = T::from(0.0001).unwrap();

        let v1 = from_vec.normalized();
        let v2 = to_vec.normalized();

        let dot = Vec3::dot_product(v1, v2);

        if dot >= threshold {
            return Self::new(one, zero, zero, zero);
        }

        if dot <= -threshold {
            let mut axis = Vec3::cross_product(Vec3::new(one, zero, zero), v1);
            if axis.length() < small {
                axis = Vec3::cross_product(Vec3::new(zero, one, zero), v1);
            }
            axis = axis.normalized();
            return Self::new(zero, axis.x, axis.y, axis.z);
        }

        let axis = Vec3::cross_product(v1, v2);
        let w = (v1.length_squared() * v2.length_squared()).sqrt() + dot;

        Self::new(w, axis.x, axis.y, axis.z).normalized()
    }

    pub fn to_rotation_matrix(self) -> Mat4<T> {
        let q = self.normalized();

        let zero = T::zero();
        let one = T::one();
        let two = one + one;

        Mat4::from_array([
            // Column 0
            one - two * (q.y * q.y + q.z * q.z),
            two * (q.x * q.y + q.w * q.z),
            two * (q.x * q.z - q.w * q.y),
            zero,
            // Column 1
            two * (q.x * q.y - q.w * q.z),
            one - two * (q.x * q.x + q.z * q.z),
            two * (q.y * q.z + q.w * q.x),
            zero,
            // Column 2
            two * (q.x * q.z + q.w * q.y),
            two * (q.y * q.z - q.w * q.x),
            one - two * (q.x * q.x + q.y * q.y),
            zero,
            // Column 3
            zero,
            zero,
            zero,
            one,
        ])
    }

    pub fn approx_eq(&self, other: Quaternion<T>) -> bool {
        (self.w - other.w).abs() < Self::epsilon()
            && (self.x - other.x).abs() < Self::epsilon()
            && (self.y - other.y).abs() < Self::epsilon()
            && (self.z - other.z).abs() < Self::epsilon()
    }
}

impl<T: Float> Mul for Quaternion<T> {
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
