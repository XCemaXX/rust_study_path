mod dielectric;
mod diffuse_ligth;
mod lambertian;
mod metal;

use std::cell::RefCell;

use crate::{color::Color, coords::Coords, hit::HitRecord, ray::Ray};
pub use dielectric::Dielectric;
pub use diffuse_ligth::DiffuseLight;
pub use lambertian::Lambertian;
pub use metal::Metal;
use rand::Rng;

pub trait Material: Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        None
    }

    fn emitted(&self, u: f32, v: f32, p: Coords) -> Color {
        Color::default()
    }
}

fn reflect(v: Coords, n: Coords) -> Coords {
    v - 2.0 * v.dot(n) * n
}
