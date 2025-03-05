mod lambertian;
mod metal;
mod dielectric;

use std::{cell::RefCell, ops::Mul};

use crate::{color::Color, coords::Coords, hit::HitRecord, ray::Ray, vec3::Vec3};
pub use lambertian::Lambertian;
pub use metal::Metal;
pub use dielectric::Dielectric;
use rand::{rngs::SmallRng, Rng};

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)>;
}

pub type Albedo = Vec3<AlbedoTag>;

#[derive(Default, Clone, Copy)]
pub struct AlbedoTag;

impl Mul<Color> for Albedo {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(
            self.0 * rhs.0,
            self.1 * rhs.1,
            self.2 * rhs.2,
        )
    }
}

fn reflect(v: Coords, n: Coords) -> Coords {
    v - 2.0 * v.dot(n) * n
}

fn random_in_unit_sphere(rng: &mut SmallRng) -> Coords {
    loop {
        let p = 2.0
            * Coords::new(rng.random::<f32>(), rng.random::<f32>(), rng.random::<f32>()) 
            - Coords::new(1.0, 1.0, 1.0);
        break p;
    }
}