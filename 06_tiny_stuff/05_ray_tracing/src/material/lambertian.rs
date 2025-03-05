use super::*;
use std::cell::RefCell;
use rand::{rngs::SmallRng, SeedableRng};

pub struct Lambertian {
    albedo: Albedo,
    rng: RefCell<SmallRng>,
}

impl Lambertian {
    pub fn new(albedo: Albedo) -> Self {
        Self {
            albedo,
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng()))
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)> {
        let random = random_in_unit_sphere(&mut self.rng.borrow_mut());
        let target = rec.p + rec.normal + random;
        let scattered = Ray::new(rec.p, target - rec.p);
        Some((
            scattered,
            self.albedo
        ))
    }
}