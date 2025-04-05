use std::{f32::consts::PI, sync::Arc};

use crate::{
    Color,
    hit::HitRecord,
    pdf::SpherePdf,
    ray::Ray,
    texture::{IntoSharedTexture, SolidColor, Texture},
};

use super::{Material, ScatterResult, ScatterType};

pub struct Isotropic {
    texture: Arc<dyn Texture>,
}

impl Isotropic {
    #[allow(dead_code)]
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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
        let attenuation = self.texture.value(rec.u, rec.v, rec.p);
        let pdf = Box::new(SpherePdf::new());
        Some(ScatterResult {
            attenuation,
            scattered: ScatterType::Diffuse { pdf },
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f32 {
        1. / (4. * PI)
    }
}
