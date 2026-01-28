use super::quaternion::Quaternion;
use super::vector::Vec3;
use num_traits::Float;
use std::ops::Mul;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mat4<T: Float> {
    pub data: [T; 16],
}

impl<T: Float> Mat4<T> {
    fn identity_data() -> [T; 16] {
        let zero = T::zero();
        let one = T::one();
        [
            one, zero, zero, zero, zero, one, zero, zero, zero, zero, one, zero, zero, zero, zero, one,
        ]
    }

    pub fn new() -> Self {
        Self {
            data: Self::identity_data(),
        }
    }

    pub fn from_array(data: [T; 16]) -> Self {
        Self { data }
    }

    pub fn set_to_identity(&mut self) {
        self.data = Self::identity_data()
    }

    pub fn translate(&mut self, vector: Vec3<T>) {
        let mut mat4 = Self::new();
        mat4.data[12] = vector.x;
        mat4.data[13] = vector.y;
        mat4.data[14] = vector.z;

        *self = *self * mat4
    }

    pub fn scale(&mut self, vector: Vec3<T>) {
        let mut mat4 = Self::new();
        mat4.data[0] = vector.x;
        mat4.data[5] = vector.y;
        mat4.data[10] = vector.z;

        *self = *self * mat4
    }

    pub fn rotate(&mut self, quat: Quaternion<T>) {
        *self = *self * quat.to_rotation_matrix()
    }

    pub fn look_at(&mut self, eye: Vec3<T>, center: Vec3<T>, up: Vec3<T>) {
        let zero = T::zero();
        let one = T::one();

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

        self.data[3] = zero;
        self.data[7] = zero;
        self.data[11] = zero;
        self.data[15] = one;
    }

    pub fn perspective(&mut self, fov: T, aspect: T, near_plane: T, far_plane: T) {
        self.set_to_identity();

        let zero = T::zero();
        let one = T::one();
        let two = one + one;

        let rad = fov.to_radians();
        let f = one / (rad / two).tan();

        self.data[0] = f / aspect;
        self.data[5] = f;
        self.data[10] = (far_plane + near_plane) / (near_plane - far_plane);
        self.data[11] = -one;
        self.data[14] = (two * far_plane * near_plane) / (near_plane - far_plane);
        self.data[15] = zero;
    }

    pub fn ortho(&mut self, left: T, right: T, bottom: T, top: T, near_plane: T, far_plane: T) {
        self.set_to_identity();

        let one = T::one();
        let two = one + one;

        let width = right - left;
        let height = top - bottom;
        let depth = far_plane - near_plane;

        self.data[0] = two / width;
        self.data[5] = two / height;
        self.data[10] = -two / depth;
        self.data[12] = -(right + left) / width;
        self.data[13] = -(top + bottom) / height;
        self.data[14] = -(far_plane + near_plane) / depth;
    }
}

impl<T: Float> Mul for Mat4<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let mut data = Self::identity_data();

        for col in 0..4 {
            for row in 0..4 {
                let mut sum = T::zero();
                for i in 0..4 {
                    let a = self.data[i * 4 + row];
                    let b = other.data[col * 4 + i];
                    sum = sum + a * b;
                }
                data[col * 4 + row] = sum;
            }
        }
        Self::from_array(data)
    }
}
