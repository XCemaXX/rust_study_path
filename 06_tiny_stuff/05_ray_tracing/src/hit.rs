use crate::material::Material;
use crate::ray::Ray;
use crate::Coords;

pub struct HitRecord<'a> {
    pub t: f32,
    pub p: Coords,
    pub normal: Coords,
    pub material: &'a dyn Material,
}

pub trait Hit {
    fn hit(&self, r: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord>;
}

pub type HitableList = Vec<Box<dyn Hit>>;

impl Hit for HitableList {
    fn hit(&self, r: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut hit_anything = None;
        let mut closest_so_far = tmax;
        for obj in self {
            let temp_rec = obj.hit(r, tmin, closest_so_far);
            if let Some(rec) = temp_rec {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }
        hit_anything
    }
}