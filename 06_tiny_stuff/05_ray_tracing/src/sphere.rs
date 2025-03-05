use std::ops::Range;

use crate::Coords;
use crate::hit;
use crate::hit::HitRecord;
use crate::material::Material;

#[derive(Default)]
pub struct Sphere<T: Material> {
    center: Coords,
    radius: f32,
    material: T,
}

impl<T: Material> Sphere<T> {
    pub fn new(center: Coords, radius: f32, material: T) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl<T: Material> hit::Hit for Sphere<T> {
    fn hit(&self, r: &crate::ray::Ray, ray_t: Range<f32>) -> Option<hit::HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().dot(r.direction());
        let h = oc.dot(r.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant > 0.0 {
            let sqrtd = discriminant.sqrt();
            for t in [(-h - sqrtd) / a, (-h + sqrtd) / a] {
                if ray_t.contains(&t) {
                    let p = r.at(t);
                    let outward_normal = (p - self.center) / self.radius;
                    return Some(
                        HitRecord::new(t, p, outward_normal, &self.material)
                            .set_face_normal(r, outward_normal),
                    );
                }
            }
        }
        None
    }
}
