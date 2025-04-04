use crate::texture::{IntoSharedTexture, SolidColor, Texture};

use super::*;
use rand::{SeedableRng, rngs::SmallRng};
use std::{cell::RefCell, f32::consts::PI, sync::Arc};

thread_local! {
    static LAMBERTIAN_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Lambertian {
    texture: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn from_color(albedo: Color) -> Self {
        Self {
            texture: Arc::new(SolidColor::new(albedo)),
        }
    }

    pub fn from_texture<T: IntoSharedTexture>(texture: T) -> Self {
        Self {
            texture: texture.into_arc(),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction = LAMBERTIAN_RNG
            .with(|rng| Coords::random_on_hemisphere(rec.normal, &mut rng.borrow_mut()));
        
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        };
        let scattered = Ray::new_timed(rec.p, scatter_direction, r_in.time());
        let attenuation = self.texture.value(rec.u, rec.v, rec.p);
        Some((scattered, attenuation))
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f32 {
        1. / (2. * PI)
    }
}
