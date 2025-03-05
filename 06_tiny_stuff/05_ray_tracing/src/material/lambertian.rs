use super::*;
use rand::{SeedableRng, rngs::SmallRng};
use std::cell::RefCell;

thread_local! {
    static LAMBERTIAN_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Lambertian {
    albedo: Albedo,
}

impl Lambertian {
    pub fn new(albedo: Albedo) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)> {
        let mut scatter_direction = LAMBERTIAN_RNG
            .with(|rng| rec.normal + Coords::random_unit_vector(&mut rng.borrow_mut()));
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        };
        let scattered = Ray::new(rec.p, scatter_direction);
        Some((scattered, self.albedo))
    }
}
