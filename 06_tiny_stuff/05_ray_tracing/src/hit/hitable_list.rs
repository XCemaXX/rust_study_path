use std::ops::Range;

use crate::ray::Ray;

use super::{Aabb, Hit, HitRecord};

pub struct HitableList {
    objects: Vec<Box<dyn Hit>>,
    bbox: Aabb,
}

impl HitableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::empty(),
        }
    }

    pub fn push(&mut self, object: impl Hit + 'static) {
        let bbox = object.bounding_box().clone();
        self.objects.push(Box::new(object));
        self.bbox = Aabb::from_boxes(self.bbox.clone(), bbox);
    }

    pub fn take_objects(self) -> Vec<Box<dyn Hit>> {
        self.objects
    }
}

impl Hit for HitableList {
    fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
        let mut hit_anything = None;
        let mut closest_so_far = ray_t.end;
        for obj in &self.objects {
            let temp_rec = obj.hit(r, ray_t.start..closest_so_far);
            if let Some(rec) = temp_rec {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }
        hit_anything
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

impl FromIterator<Box<dyn Hit>> for HitableList {
    fn from_iter<T: IntoIterator<Item = Box<dyn Hit>>>(iter: T) -> Self {
        let objects: Vec<Box<dyn Hit>> = iter.into_iter().collect();
        let bbox = objects.iter().fold(Aabb::empty(), |acc, object| {
            Aabb::from_boxes(acc, object.bounding_box().clone())
        });
        Self {
            objects,
            bbox
        }
    }
}