use std::{cell::RefCell, ops::Range, sync::Arc};

use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    Color,
    coords::Coords,
    hit::{Aabb, Hit, HitRecord},
    material::{IntoSharedMaterial, Isotropic, Material},
    ray::Ray,
    texture::{IntoSharedTexture, SolidColor},
};

thread_local! {
    static CONSTANT_MEDIUM_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct ConstantMedium<T: Hit> {
    boundary: T,
    neg_inc_density: f32,
    phase_function: Arc<dyn Material>,
}

impl<H: Hit> ConstantMedium<H> {
    pub fn from_color(boundary: H, density: f32, albedo: Color) -> Self {
        Self::from_texture::<SolidColor>(boundary, density, albedo.into())
    }

    pub fn from_texture<T: IntoSharedTexture>(boundary: H, density: f32, texture: T) -> Self {
        let phase_function = Isotropic::from_texture(texture).into_arc();
        Self {
            boundary,
            neg_inc_density: -1. / density,
            phase_function,
        }
    }
}

impl<T: Hit> Hit for ConstantMedium<T> {
    fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
        let mut rec1 = self.boundary.hit(r, interval_universe())?;
        let range = (rec1.t + 0.001)..f32::MAX;
        let mut rec2 = self.boundary.hit(r, range)?;
        rec1.t = rec1.t.max(ray_t.start);
        rec2.t = rec2.t.min(ray_t.end);

        if rec1.t >= rec2.t {
            return None;
        }

        rec1.t = rec1.t.max(0.);

        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = CONSTANT_MEDIUM_RNG
            .with(|rng| self.neg_inc_density * f32::ln(rng.borrow_mut().random()));
        if hit_distance > distance_inside_boundary {
            return None;
        }

        let rec_t = rec1.t + hit_distance / ray_length;
        let rec = HitRecord::new(
            rec_t,
            r.at(rec_t),
            Coords::new(1., 0., 0.),
            self.phase_function.as_ref(),
        );
        Some(rec)
    }

    fn bounding_box(&self) -> &Aabb {
        self.boundary.bounding_box()
    }
}

fn interval_universe() -> Range<f32> {
    f32::MIN..f32::MAX
}
