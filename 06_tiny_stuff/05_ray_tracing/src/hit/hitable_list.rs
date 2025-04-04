use std::{cell::RefCell, ops::Range};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{coords::Coords, pdf::PdfWithOrigin, ray::Ray};

use super::{Aabb, Hit, HitRecord};

thread_local! {
    static HITABBLE_LIST_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct HitableList<T: ?Sized> {
    objects: Vec<Box<T>>,
    bbox: Aabb,
}

impl<T: ?Sized> HitableList<T> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::empty(),
        }
    }
}

impl HitableList<dyn Hit> {
    pub fn push(&mut self, object: impl Hit + 'static) {
        let bbox = object.bounding_box().clone();
        self.objects.push(Box::new(object));
        self.bbox = Aabb::from_boxes(self.bbox.clone(), bbox);
    }

    pub fn take_objects(self) -> Vec<Box<dyn Hit>> {
        self.objects
    }
}

impl HitableList<dyn PdfWithOrigin> {
    pub fn push(&mut self, object: impl PdfWithOrigin + Hit + 'static) {
        let bbox = object.bounding_box().clone();
        self.objects.push(Box::new(object));
        self.bbox = Aabb::from_boxes(self.bbox.clone(), bbox);
    }
}


impl<T: ?Sized + Hit> Hit for HitableList<T> {
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

impl PdfWithOrigin for HitableList<dyn PdfWithOrigin> {
    fn pdf_value(&self, origin: Coords, direction: Coords) -> f32 {
        let weight = 1.0 / self.objects.len() as f32;
        let sum = self.objects.iter().fold(0., 
            |acc, o| {
                acc + weight * o.pdf_value(origin, direction)
            });
        sum
    }

    fn random(&self, origin: Coords) -> Coords {
        HITABBLE_LIST_RNG.with(|rng| {
            let i = rng.borrow_mut().random_range(0..self.objects.len());
            self.objects[i].random(origin)
        }) 
    }
}

impl FromIterator<Box<dyn Hit>> for HitableList<dyn Hit> {
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