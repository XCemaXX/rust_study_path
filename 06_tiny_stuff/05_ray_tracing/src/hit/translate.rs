use crate::{coords::Coords, ray::Ray};

use super::{Aabb, Hit, HitRecord};

pub struct Translate<T: Hit> {
    object: T,
    offset: Coords,
    bbox: Aabb,
}

impl<T: Hit> Translate<T> {
    pub fn new(object: T, offset: Coords) -> Self {
        let bbox = object.bounding_box().clone() + offset;
        Self {
            object,
            offset,
            bbox,
        }
    }
}

impl<T: Hit> Hit for Translate<T> {
    fn hit(&self, r: &crate::ray::Ray, ray_t: std::ops::Range<f32>) -> Option<HitRecord> {
        // Move the ray backwards by the offset
        let offset_r = Ray::new_timed(r.origin() - self.offset, r.direction(), r.time());
        // Determine whether an intersection exists along the offset ray (and if so, where)
        let mut rec = self.object.hit(&offset_r, ray_t)?;
        rec.p += self.offset;
        Some(rec)
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}
