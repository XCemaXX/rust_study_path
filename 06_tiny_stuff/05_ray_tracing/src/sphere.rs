use std::ops::Range;

use crate::Coords;
use crate::hit::{self, Aabb, HitRecord};
use crate::material::Material;
use crate::ray::Ray;

pub struct Sphere<T: Material> {
    center: Ray,
    radius: f32,
    material: T,
    bbox: Aabb,
}

impl<T: Material> Sphere<T> {
    pub fn new(static_center: Coords, radius: f32, material: T) -> Self {
        let radius = radius.max(0.0);
        let rvec = Coords::new(radius, radius, radius);
        Self {
            center: Ray::new(static_center, Coords::new(0.0, 0.0, 0.0)),
            radius,
            material,
            bbox: Aabb::from_points(static_center - rvec, static_center + rvec),
        }
    }

    pub fn new_moving(center1: Coords, center2: Coords, radius: f32, material: T) -> Self {
        let radius = radius.max(0.0);
        let center = Ray::new(center1, center2 - center1);
        let rvec = Coords::new(radius, radius, radius);
        let box1 = Aabb::from_points(center.at(0.0) - rvec, center.at(0.0) + rvec);
        let box2 = Aabb::from_points(center.at(1.0) - rvec, center.at(1.0) + rvec);
        Self {
            center,
            radius,
            material,
            bbox: Aabb::from_boxes(box1, box2),
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

    fn bounding_box(&self) -> Aabb {
        self.bbox.clone()
    }
}
