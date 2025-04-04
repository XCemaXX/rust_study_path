use core::f32;
use std::f32::consts::PI;

use crate::{coords::{Axis, Coords}, ray::Ray};

use super::{Aabb, Hit, HitRecord};

pub struct RotateY<T: Hit> {
    object: T,
    sin_theta: f32,
    cos_theta: f32,
    bbox: Aabb,
}

impl<T: Hit> RotateY<T> {
    pub fn new(object: T, angle: f32) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = f32::sin(radians);
        let cos_theta = f32::cos(radians);
        let bbox = object.bounding_box().clone();

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for corner in bbox.corners() {
            let x = corner.x();
            let z = corner.z();
            let newx = cos_theta * x + sin_theta * z;
            let newz = -sin_theta * x + cos_theta * z;
            let tester = Coords::new(newx, corner.y(), newz);

            for axis in [Axis::X, Axis::Y, Axis::Z].into_iter() {
                min[axis as usize] = min[axis as usize].min(tester[axis]);
                max[axis as usize] = max[axis as usize].max(tester[axis]);
            }
        }

        Self {
            object,
            sin_theta,
            cos_theta,
            bbox: Aabb::from_points(Coords::from(min), Coords::from(max)),
        }
    }
}

impl<T: Hit> Hit for RotateY<T> {
    fn hit(&self, r: &crate::ray::Ray, ray_t: std::ops::Range<f32>) -> Option<HitRecord> {
        // Transform the ray from world space to object space.
        let origin = Coords::new(
            (self.cos_theta * r.origin().x()) - (self.sin_theta * r.origin().z()),
            r.origin().y(),
            (self.sin_theta * r.origin().x()) + (self.cos_theta * r.origin().z()),
        );

        let direction = Coords::new(
            (self.cos_theta * r.direction().x()) - (self.sin_theta * r.direction().z()),
            r.direction().y(),
            (self.sin_theta * r.direction().x()) + (self.cos_theta * r.direction().z()),
        );

        let rotated_r = Ray::new_timed(origin, direction, r.time());
        let mut rec = self.object.hit(&rotated_r, ray_t)?;

        // Transform the intersection from object space back to world space.

        rec.p = Coords::new(
            (self.cos_theta * rec.p.x()) + (self.sin_theta * rec.p.z()),
            rec.p.y(),
            (-self.sin_theta * rec.p.x()) + (self.cos_theta * rec.p.z()),
        );

        rec.normal = Coords::new(
            (self.cos_theta * rec.normal.x()) + (self.sin_theta * rec.normal.z()),
            rec.normal.y(),
            (-self.sin_theta * rec.normal.x()) + (self.cos_theta * rec.normal.z()),
        );

        Some(rec)
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.
}
