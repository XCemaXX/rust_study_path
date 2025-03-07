mod dielectric;
mod lambertian;
mod metal;

use std::{cell::RefCell, ops::Mul};

use crate::{color::Color, coords::Coords, hit::HitRecord, ray::Ray, vec3::Vec3};
pub use dielectric::Dielectric;
pub use lambertian::Lambertian;
pub use metal::Metal;
use rand::Rng;

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)>;
}

pub type Albedo = Vec3<AlbedoTag>;

#[derive(Default, Clone, Copy)]
pub struct AlbedoTag;

impl Mul<Color> for Albedo {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(self.0 * rhs.r(), self.1 * rhs.g(), self.2 * rhs.b())
    }
}

fn reflect(v: Coords, n: Coords) -> Coords {
    v - 2.0 * v.dot(n) * n
}
