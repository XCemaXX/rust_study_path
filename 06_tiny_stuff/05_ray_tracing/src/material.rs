mod dielectric;
mod diffuse_ligth;
mod isotropic;
mod lambertian;
mod metal;

use std::{cell::RefCell, sync::Arc};

use crate::{color::Color, coords::Coords, hit::HitRecord, pdf::Pdf, ray::Ray};
pub use dielectric::Dielectric;
pub use diffuse_ligth::DiffuseLight;
pub use isotropic::Isotropic;
pub use lambertian::Lambertian;
pub use metal::Metal;
use rand::Rng;

pub enum ScatterType {
    Diffuse { pdf: Box<dyn Pdf> },
    Specular { ray: Ray },
}

pub struct ScatterResult {
    pub attenuation: Color,
    pub scattered: ScatterType,
}

pub trait Material: Sync + Send {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<ScatterResult> {
        None
    }

    fn emitted(&self, _r_in: &Ray, _rec: &HitRecord, _u: f32, _v: f32, _p: Coords) -> Color {
        Color::default()
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f32 {
        0.0
    }
}

fn reflect(v: Coords, n: Coords) -> Coords {
    v - 2. * v.dot(n) * n
}

pub trait IntoSharedMaterial {
    fn into_arc(self) -> Arc<dyn Material>;
}

impl IntoSharedMaterial for Arc<dyn Material> {
    fn into_arc(self) -> Arc<dyn Material> {
        self
    }
}

impl<T: Material + 'static> IntoSharedMaterial for T {
    fn into_arc(self) -> Arc<dyn Material> {
        Arc::new(self)
    }
}
