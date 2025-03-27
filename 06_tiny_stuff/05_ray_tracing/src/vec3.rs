use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Range, Sub};

use rand::Rng;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec3<Tag>(
    pub(crate) f32,
    pub(crate) f32,
    pub(crate) f32,
    PhantomData<Tag>,
);

impl<Tag> Vec3<Tag> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(x, y, z, PhantomData)
    }

    pub fn random1(rng: &mut impl Rng) -> Self {
        Self(rng.random(), rng.random(), rng.random(), PhantomData)
    }

    pub fn random(rng: &mut impl Rng, range: Range<f32>) -> Self {
        Self(
            rng.random_range(range.clone()),
            rng.random_range(range.clone()),
            rng.random_range(range),
            PhantomData,
        )
    }

    pub fn unit_vector(self) -> Self {
        let l = self.length();
        assert_ne!(l, 0.0);
        self / l
    }

    pub fn random_unit_vector(rng: &mut impl Rng) -> Self {
        loop {
            let p = Self::random(rng, -1.0..1.0);
            let lensq = p.length_squared();
            if 1e-160 < lensq && lensq <= 1.0 {
                return p / f32::sqrt(lensq);
            }
        }
    }

    pub fn cross(self, other: Self) -> Self {
        Vec3::new(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }

    pub fn dot(self, other: Self) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    pub fn sqrt_axis(self) -> Self {
        fn sqrt_pos(val: f32) -> f32 {
            if val > 0.0 { f32::sqrt(val) } else { 0.0 }
        }

        Self::new(sqrt_pos(self.0), sqrt_pos(self.1), sqrt_pos(self.2))
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        f32::abs(self.0) < s && f32::abs(self.1) < s && f32::abs(self.2) < s
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
        Vec3::new(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl<Tag> Mul for Vec3<Tag> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::new(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

impl<Tag> Div for Vec3<Tag> {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        assert_ne!(other.0, 0.0);
        assert_ne!(other.1, 0.0);
        assert_ne!(other.2, 0.0);
        Self::new(self.0 / other.0, self.1 / other.1, self.2 / other.2)
    }
}

impl<Tag> Neg for Vec3<Tag> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.0, -self.1, -self.2)
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
        assert_ne!(scalar, 0.0);
        tmp /= scalar;
        tmp
    }
}

impl<Tag> DivAssign<f32> for Vec3<Tag> {
    fn div_assign(&mut self, scalar: f32) {
        assert_ne!(scalar, 0.0);
        self.0 /= scalar;
        self.1 /= scalar;
        self.2 /= scalar;
    }
}

impl<Tag> ToString for Vec3<Tag> {
    fn to_string(&self) -> String {
        format!("{} {} {}", self.0 as i32, self.1 as i32, self.2 as i32)
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

impl<Tag> From<[f32; 3]> for Vec3<Tag> {
    fn from(v: [f32; 3]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl<Tag> Index<Axis> for Vec3<Tag> {
    type Output = f32;

    fn index(&self, index: Axis) -> &Self::Output {
        use Axis::*;
        match index {
            X => &self.0,
            Y => &self.1,
            Z => &self.2,
        }
    }
}
