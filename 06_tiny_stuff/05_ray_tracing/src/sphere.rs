use std::ops::Range;

use crate::ray::Ray;
use crate::Coords;
use crate::hit;
use crate::hit::HitRecord;
use crate::material::Material;

#[derive(Default)]
pub struct Sphere<T: Material> {
    center: Ray,
    radius: f32,
    material: T,
}

impl<T: Material> Sphere<T> {
    pub fn new(static_center: Coords, radius: f32, material: T) -> Self {
        Self {
            center: Ray::new(static_center, Coords::new(0.0, 0.0, 0.0)),
            radius,
            material,
        }
    }

    pub fn new_moving(center1: Coords, center2: Coords, radius: f32, material: T) -> Self {
        Self {
            center: Ray::new(center1, center2 - center1),
            radius,
            material,
        }
    }
}

impl<T: Material + Send + Sync> hit::Hit for Sphere<T> {
    fn hit(&self, r: &crate::ray::Ray, ray_t: Range<f32>) -> Option<hit::HitRecord> {
        let current_center = self.center.at(r.time());
        let oc = r.origin() - current_center;
        let a = r.direction().dot(r.direction());
        let h = oc.dot(r.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant > 0.0 {
            let sqrtd = discriminant.sqrt();
            for t in [(-h - sqrtd) / a, (-h + sqrtd) / a] {
                if ray_t.contains(&t) {
                    let p = r.at(t);
                    let outward_normal = (p - current_center) / self.radius;
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
