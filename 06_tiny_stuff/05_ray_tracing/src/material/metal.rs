use rand::{SeedableRng, rngs::SmallRng};

use super::*;
pub struct Metal {
    albedo: Albedo,
    fuzz: f32,
    rng: RefCell<SmallRng>,
}

impl Metal {
    pub fn new(albedo: Albedo, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng())),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)> {
        let reflected = reflect(r_in.direction(), rec.normal).unit_vector();
        let reflected =
            reflected + self.fuzz * Coords::random_unit_vector(&mut self.rng.borrow_mut());
        let scattered = Ray::new(rec.p, reflected);
        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((scattered, self.albedo))
        } else {
            None
        }
    }
}
