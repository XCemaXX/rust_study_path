use rand::{rngs::SmallRng, SeedableRng};

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
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng()))
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)> {
        let reflected = reflect(r_in.direction().unit_vector(), rec.normal);
        let random = random_in_unit_sphere(&mut self.rng.borrow_mut());
        let scattered = Ray::new(rec.p, reflected + self.fuzz * random);
        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((
                scattered,
                self.albedo
            ))
        } else {
            None
        }
    }
}