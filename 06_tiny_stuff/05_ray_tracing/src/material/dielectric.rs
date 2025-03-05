use super::*;
use rand::{rngs::SmallRng, SeedableRng};

pub struct Dielectric {
    ref_idx: f32,
    rng: RefCell<SmallRng>,
}

impl Dielectric {
    pub fn new(ref_idx: f32) -> Self {
        Self { 
            ref_idx,
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng()))
        }
    }
}

fn refract(v: Coords, n: Coords, ni_over_nt: f32) -> Option<Coords> {
    let uv = v.unit_vector();
    let dt = uv.dot(n);
    let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
    if discriminant > 0.0 {
        Some( ni_over_nt * (uv - n * dt) - n * f32::sqrt(discriminant))
    } else {
        None
    }
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * f32::powi(1.0 - cosine, 5)
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Albedo)> {
        let r_dir = r_in.direction();
        let attenuation = Albedo::new(1.0, 1.0, 1.0);
        let cosine = r_dir.dot(rec.normal) / r_dir.length();
        let (outward_normal, ni_over_nt, cosine) = if r_dir.dot(rec.normal) > 0.0 {
            (-rec.normal, self.ref_idx, self.ref_idx * cosine)
        } else {
            (rec.normal, 1.0 / self.ref_idx, -cosine)
        };

        let refracted = refract(r_dir, outward_normal, ni_over_nt);
        let (refracted, reflect_prob) = if refracted.is_some() {
            (refracted.unwrap(), schlick(cosine, self.ref_idx))
        } else {
            (Coords::default(), 1.0)
        };
        let scattered = if self.rng.borrow_mut().random::<f32>() < reflect_prob {
            let reflected = reflect(r_dir, rec.normal);
            Ray::new(rec.p, reflected)
        } else {
            Ray::new(rec.p, refracted)
        };
        Some((scattered, attenuation))
    }
}