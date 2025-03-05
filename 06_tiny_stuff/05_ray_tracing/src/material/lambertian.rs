use super::*;
use rand::{SeedableRng, rngs::SmallRng};
use std::cell::RefCell;

pub struct Lambertian {
    albedo: Albedo,
    rng: RefCell<SmallRng>,
}

impl Lambertian {
    pub fn new(albedo: Albedo) -> Self {
        Self {
            albedo,
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng())),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)> {
        let mut scatter_direction =
            rec.normal + Coords::random_unit_vector(&mut self.rng.borrow_mut());
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        };
        let scattered = Ray::new(rec.p, scatter_direction);
        Some((scattered, self.albedo))
    }
}
