use crate::{
    pdf::CosinePdf,
    texture::{IntoSharedTexture, SolidColor, Texture},
};

use super::*;
use std::{f32::consts::PI, sync::Arc};

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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
        let attenuation = self.texture.value(rec.u, rec.v, rec.p);
        let pdf = Box::new(CosinePdf::new(rec.normal));
        Some(ScatterResult {
            attenuation,
            scattered: ScatterType::Diffuse { pdf },
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f32 {
        let cos_theta = rec.normal.dot(Coords::unit_vector(scattered.direction()));
        if cos_theta < 0.0 { 0. } else { cos_theta / PI }
    }
}
