use crate::texture::{SolidColor, Texture};

use super::*;
use rand::{SeedableRng, rngs::SmallRng};
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static LAMBERTIAN_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Lambertian {
    texture: Arc<Box<dyn Texture>>,
}

impl Lambertian {
    pub fn from_color(albedo: Color) -> Self {
        Self {
            texture: Arc::new(Box::new(SolidColor::new(albedo))),
        }
    }

    pub fn from_shared_texture(texture: Arc<Box<dyn Texture>>) -> Self {
        Self { texture }
    }

    pub fn from_texture(texture: Box<dyn Texture>) -> Self {
        Self {
            texture: Arc::new(texture),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction = LAMBERTIAN_RNG
            .with(|rng| rec.normal + Coords::random_unit_vector(&mut rng.borrow_mut()));
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        };
        let scattered = Ray::new_timed(rec.p, scatter_direction, r_in.time());
        let attenuation = self.texture.value(rec.u, rec.v, rec.p);
        Some((scattered, attenuation))
    }
}
