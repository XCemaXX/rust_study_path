use std::ops::Range;

use crate::Coords;
use crate::hit::{self, Aabb, HitRecord};
use crate::material::Material;
use crate::ray::Ray;


pub struct Quad<T: Material> {
    q: Coords,
    u: Coords,
    v: Coords,
    w: Coords,
    material: T,
    bbox: Aabb,
    normal: Coords,
    d: f32,
}

impl<T: Material> Quad<T> {
    pub fn new(q: Coords, u: Coords, v: Coords, material: T) -> Self {
        let n = u.cross(v);
        let normal = n.unit_vector();
        let d = normal.dot(q);
        let w = n / n.dot(n);
        let bbox_diagonal1 = Aabb::from_points(q, q + u + v);
        let bbox_diagonal2 = Aabb::from_points(q + u, q + v);
        let bbox = Aabb::from_boxes(bbox_diagonal1, bbox_diagonal2);

        Self {
            q,
            u,
            v,
            w,
            material,
            bbox,
            normal,
            d,
        }
    }
}

impl<T: Material> hit::Hit for Quad<T> {
    fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
        let denom = self.normal.dot(r.direction());

        if f32::abs(denom) < 1e-8 {
            return None;
        }

        let t = (self.d - self.normal.dot(r.origin())) / denom;
        if !ray_t.contains(&t) {
            return None;
        }
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(planar_hitpt_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitpt_vector));
        if !is_interior(alpha, beta) {
            return None;
        }
        let rec = HitRecord::new(t, intersection, self.normal, &self.material);
        let mut rec = rec.set_face_normal(r, self.normal);
        rec.u = alpha;
        rec.v = beta;
        Some(rec)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox.clone()
    }
}

fn is_interior(a: f32, b: f32) -> bool {
    let unit_interval = 0.0..1.0;
    unit_interval.contains(&a) && unit_interval.contains(&b)
}