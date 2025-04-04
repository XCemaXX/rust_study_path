use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::pdf::PdfWithOrigin;
use crate::Coords;
use crate::hit::{self, Aabb, Hit, HitRecord};
use crate::material::{IntoSharedMaterial, Material};
use crate::ray::Ray;

thread_local! {
    static QUAD_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Quad {
    q: Coords,
    u: Coords,
    v: Coords,
    w: Coords,
    material: Arc<dyn Material>,
    bbox: Aabb,
    normal: Coords,
    d: f32,
    area: f32,
}

impl Quad {
    pub fn new<M: IntoSharedMaterial>(q: Coords, u: Coords, v: Coords, material: M) -> Self {
        let material = material.into_arc();
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
            area: n.length(),
        }
    }
}

impl hit::Hit for Quad {
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
        let rec = HitRecord::new(t, intersection, self.normal, self.material.as_ref());
        let mut rec = rec.set_face_normal(r, self.normal);
        rec.u = alpha;
        rec.v = beta;
        Some(rec)
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

impl PdfWithOrigin for Quad {
    fn pdf_value(&self, origin: Coords, direction: Coords) -> f32 {
        let ray = Ray::new(origin, direction);
        let Some(rec) = self.hit(&ray, 0.001..f32::MAX) else {
            return 0.001; //todo 0.
        };

        let distance_squared = rec.t * rec.t * direction.length_squared();
        let cosine = f32::abs(direction.dot(rec.normal) / direction.length());

        distance_squared / (cosine * self.area)
    }

    fn random(&self, origin: Coords) -> Coords {
        let p = QUAD_RNG.with(|rng| {
            let r1 = rng.borrow_mut().random::<f32>();
            let r2 = rng.borrow_mut().random::<f32>();
            self.q + (r1 * self.u) + (r2 * self.v)
        });
        p - origin
    }
}

fn is_interior(a: f32, b: f32) -> bool {
    let unit_interval = 0.0..1.0;
    unit_interval.contains(&a) && unit_interval.contains(&b)
}
