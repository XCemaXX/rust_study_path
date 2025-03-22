mod aabb;
mod bvh;
mod hitable_list;

use std::ops::Range;

use crate::Coords;
use crate::material::Material;
use crate::ray::Ray;
pub use aabb::Aabb;
pub use bvh::BvhNode;
pub use hitable_list::HitableList;

pub struct HitRecord<'a> {
    pub p: Coords,
    pub normal: Coords,
    pub material: &'a dyn Material,
    pub t: f32,
    pub u: f32,
    pub v: f32,
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
            p,
            normal,
            material,
            t,
            u: 0.0,
            v: 0.0,
            front_face: true,
        }
    }
}

pub trait Hit: Send + Sync {
    fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord>;

    fn bounding_box(&self) -> Aabb;
}
