use std::ops::Range;

use crate::Coords;
use crate::material::Material;
use crate::ray::Ray;

pub struct HitRecord<'a> {
    pub t: f32,
    pub p: Coords,
    pub normal: Coords,
    pub material: &'a dyn Material,
    pub front_face: bool,
}

impl HitRecord<'_> {
    pub fn set_face_normal(self, r: &Ray, outward_normal: Coords) -> Self {
        // Sets the hit record normal vector.
        // NOTE: the parameter `outward_normal` is assumed to have unit length.
        let mut hit = self;
        hit.front_face = r.direction().dot(outward_normal) < 0.0;
        hit.normal = if hit.front_face {
            outward_normal
        } else {
            -outward_normal
        };
        hit
    }

    pub fn new<'a>(t: f32, p: Coords, normal: Coords, material: &'a dyn Material) -> HitRecord<'a> {
        HitRecord {
            t,
            p,
            normal,
            material,
            front_face: true,
        }
    }
}

pub trait Hit {
    fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord>;
}

pub type HitableList = Vec<Box<dyn Hit>>;

impl Hit for HitableList {
    fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
        let mut hit_anything = None;
        let mut closest_so_far = ray_t.end;
        for obj in self {
            let temp_rec = obj.hit(r, ray_t.start..closest_so_far);
            if let Some(rec) = temp_rec {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }
        hit_anything
    }
}
