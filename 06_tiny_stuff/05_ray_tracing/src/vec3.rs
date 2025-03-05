use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub};
use std::marker::PhantomData;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec3<Tag>(pub(crate) f32, pub(crate) f32, pub(crate) f32, PhantomData<Tag>);


impl<Tag> Vec3<Tag> {
    pub fn new(x: f32, y: f32, z: f32) -> Self { 
        Self(x, y, z, PhantomData) 
    }

    pub fn unit_vector(self) -> Self {
        let l = self.length();
        self / l
    }

    pub fn cross(self, other: Self) -> Self {
        Vec3::new(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0)
    }

    pub fn dot(self, other: Self) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    pub fn length(&self) -> f32 {
        self.squared_length().sqrt()
    }

    pub fn squared_length(&self) -> f32 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    pub fn sqrt_axis(self) -> Self {
        Self::new(
            f32::sqrt(self.0),
            f32::sqrt(self.1),
            f32::sqrt(self.2),
        )
    }
}

impl<Tag> Add for Vec3<Tag> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let mut tmp = self;
        tmp += other;
        tmp
    }
}

impl<Tag> AddAssign for Vec3<Tag> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 += other.1;
        self.2 += other.2;
    }
}

impl<Tag> Sub for Vec3<Tag> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vec3::new(
            self.0 - other.0,
            self.1 - other.1,
            self.2 - other.2
        )
    }
}

impl<Tag> Mul for Vec3<Tag> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::new(
            self.0 * other.0, 
            self.1 * other.1, 
            self.2 * other.2
        )
    }
}

impl<Tag> Div for Vec3<Tag> {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        Self::new(
            self.0 / other.0, 
            self.1 / other.1, 
            self.2 / other.2
        )
    }
}

impl<Tag> Neg for Vec3<Tag> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(
            -self.0, 
            -self.1, 
            -self.2
        )
    }
}

impl<Tag> Mul<f32> for Vec3<Tag> {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        let mut tmp = self;
        tmp *= scalar;
        tmp
    }
}

impl<Tag> Mul<Vec3<Tag>> for f32 {
    type Output = Vec3<Tag>;
    fn mul(self, other: Vec3<Tag>) -> Self::Output {
        other * self
    }
}

impl<Tag> MulAssign<f32> for Vec3<Tag> {
    fn mul_assign(&mut self, scalar: f32) {
        self.0 *= scalar; 
        self.1 *= scalar; 
        self.2 *= scalar;
    }
}

impl<Tag> Div<f32> for Vec3<Tag> {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        let mut tmp = self;
        tmp /= scalar;
        tmp
    }
}

impl<Tag> DivAssign<f32> for Vec3<Tag> {
    fn div_assign(&mut self, scalar: f32) {
        self.0 /= scalar;
        self.1 /= scalar; 
        self.2 /= scalar;
    }
}

impl<Tag> ToString for Vec3<Tag> {
    fn to_string(&self) -> String {
        format!("{} {} {}", self.0, self.1, self.2)
    }
}  


impl<Tag> From<Vec3<Tag>> for (f32, f32, f32) {
    fn from(v: Vec3<Tag>) -> Self {
        (v.0, v.1, v.2)
    }
}

impl<Tag> From<(f32, f32, f32)> for Vec3<Tag> {
    fn from(t: (f32, f32, f32)) -> Self {
        Self::new(t.0, t.1, t.2)
    }
}