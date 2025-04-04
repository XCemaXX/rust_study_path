use crate::{
    onb::Onb,
    texture::{IntoSharedTexture, SolidColor, Texture},
};

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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
        let uvw = Onb::new(rec.normal);
        let scatter_direction = LAMBERTIAN_RNG
            .with(|rng| uvw.transform(Coords::random_cosine_direction(&mut rng.borrow_mut())));

        let scattered = Ray::new_timed(rec.p, scatter_direction.unit_vector(), r_in.time());
        let attenuation = self.texture.value(rec.u, rec.v, rec.p);
        let pdf = uvw.w().dot(scattered.direction()) / PI;
        Some(ScatterResult {
            scattered,
            attenuation,
            pdf: Some(pdf),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f32 {
        let cos_theta = rec.normal.dot(Coords::unit_vector(scattered.direction()));
        if cos_theta < 0.0 {
            0.
        } else {
            cos_theta / PI
        }
    }
}
