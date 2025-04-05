use std::{f32::consts::PI, ops::Index};

use rand::Rng;

use crate::vec3::Vec3;

pub type Coords = Vec3<CoordsTag>;

#[derive(Debug, Default, Clone, Copy)]
pub struct CoordsTag;

impl Coords {
    pub fn x(&self) -> f32 {
        self.0
    }
    pub fn y(&self) -> f32 {
        self.1
    }
    pub fn z(&self) -> f32 {
        self.2
    }

    pub fn random_cosine_direction(rng: &mut impl Rng) -> Self {
        let r1: f32 = rng.random();
        let r2 = rng.random();

        let phi = 2. * PI * r1;
        Self::new(
            f32::cos(phi) * f32::sqrt(r2), 
            f32::sin(phi) * f32::sqrt(r2), 
            f32::sqrt(1. - r2))
    }

    pub fn unit_vector(self) -> Self {
        let l = self.length();
        assert_ne!(l, 0.);
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

    pub fn random_on_hemisphere(normal: Self, rng: &mut impl Rng) -> Self
    {
        let on_unit_sphere = Self::random_unit_vector(rng);
        if on_unit_sphere.clone().dot(normal) > 0. {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
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