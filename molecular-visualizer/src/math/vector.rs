use num_traits::Float;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vec3<T: Float> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Float> Vec3<T> {
    fn epsilon() -> T {
        T::from(1e-10).unwrap()
    }

    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(T::zero(), T::zero(), T::zero())
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len < Self::epsilon() {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    pub fn normalize(&mut self) {
        let len = self.length();
        if len >= Self::epsilon() {
            self.x = self.x / len;
            self.y = self.y / len;
            self.z = self.z / len;
        }
    }

    pub fn distance_to_point(&self, point: Vec3<T>) -> T {
        let dx = self.x - point.x;
        let dy = self.y - point.y;
        let dz = self.z - point.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn dot_product(v1: Vec3<T>, v2: Vec3<T>) -> T {
        v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
    }

    pub fn cross_product(v1: Vec3<T>, v2: Vec3<T>) -> Self {
        Self {
            x: v1.y * v2.z - v1.z * v2.y,
            y: v1.z * v2.x - v1.x * v2.z,
            z: v1.x * v2.y - v1.y * v2.x,
        }
    }

    pub fn normal(&self, v2: Vec3<T>, v3: Vec3<T>) -> Self {
        let edge1 = Vec3::new(v2.x - self.x, v2.y - self.y, v2.z - self.z);
        let edge2 = Vec3::new(v3.x - self.x, v3.y - self.y, v3.z - self.z);
        Vec3::cross_product(edge1, edge2).normalized()
    }

    pub fn approx_eq(&self, other: Vec3<T>) -> bool {
        let eps = Self::epsilon();
        (self.x - other.x).abs() < eps && (self.y - other.y).abs() < eps && (self.z - other.z).abs() < eps
    }
}

impl<T: Float> Add for Vec3<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Float> Sub for Vec3<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Float> Neg for Vec3<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl<T: Float> Mul<T> for Vec3<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl<T: Float> Mul<Vec3<T>> for Vec3<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl<T: Float> Div<T> for Vec3<T> {
    type Output = Self;
    fn div(self, scalar: T) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl<T: Float> AddAssign for Vec3<T> {
    fn add_assign(&mut self, other: Self) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
        self.z = self.z + other.z;
    }
}

impl<T: Float> SubAssign for Vec3<T> {
    fn sub_assign(&mut self, other: Self) {
        self.x = self.x - other.x;
        self.y = self.y - other.y;
        self.z = self.z - other.z;
    }
}

impl<T: Float> MulAssign<T> for Vec3<T> {
    fn mul_assign(&mut self, scalar: T) {
        self.x = self.x * scalar;
        self.y = self.y * scalar;
        self.z = self.z * scalar;
    }
}

impl<T: Float> MulAssign<Vec3<T>> for Vec3<T> {
    fn mul_assign(&mut self, other: Self) {
        self.x = self.x * other.x;
        self.y = self.y * other.y;
        self.z = self.z * other.z;
    }
}

impl<T: Float> DivAssign<T> for Vec3<T> {
    fn div_assign(&mut self, scalar: T) {
        self.x = self.x / scalar;
        self.y = self.y / scalar;
        self.z = self.z / scalar;
    }
}

impl<T: Float + fmt::Display> fmt::Display for Vec3<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec3D({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}
