use rand::{SeedableRng, rngs::SmallRng};

use super::*;

thread_local! {
    static METAL_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1. { fuzz } else { 1. },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let reflected = reflect(r_in.direction(), rec.normal).unit_vector();
        let reflected = METAL_RNG
            .with(|rng| reflected + self.fuzz * Coords::random_unit_vector(&mut rng.borrow_mut()));
        let scattered = Ray::new_timed(rec.p, reflected, r_in.time());
        if scattered.direction().dot(rec.normal) > 0. {
            Some((scattered, self.albedo))
        } else {
            None
        }
    }
}
