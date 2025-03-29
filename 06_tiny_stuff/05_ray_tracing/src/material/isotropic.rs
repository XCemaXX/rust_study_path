use std::{cell::RefCell, sync::Arc};

use rand::{SeedableRng, rngs::SmallRng};

use crate::{
    Color,
    coords::Coords,
    hit::HitRecord,
    ray::Ray,
    texture::{IntoSharedTexture, SolidColor, Texture},
};

use super::Material;

thread_local! {
    static ISOTROPIC_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Isotropic {
    texture: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn from_color(albedo: Color) -> Self {
        Self::from_texture(SolidColor::new(albedo))
    }

    pub fn from_texture<T: IntoSharedTexture>(texture: T) -> Self {
        Self {
            texture: texture.into_arc(),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let scattered = ISOTROPIC_RNG.with(|rng| {
            Ray::new_timed(
                rec.p,
                Coords::random_unit_vector(&mut rng.borrow_mut()),
                r_in.time(),
            )
        });
        let attenuation = self.texture.value(rec.u, rec.v, rec.p);
        Some((scattered, attenuation))
    }
}
