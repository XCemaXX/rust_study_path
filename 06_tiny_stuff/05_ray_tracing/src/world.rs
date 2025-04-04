use std::ops::Range;

use crate::{hit::{BvhNode, Hit, HitRecord, HitableList}, pdf::PdfWithOrigin, ray::Ray};


pub struct World {
    objects: HitableList<dyn Hit>,
    lights: HitableList<dyn PdfWithOrigin>,
}

impl World {
    pub fn new() -> Self {
        Self {
            objects: HitableList::<dyn Hit>::new(),
            lights: HitableList::<dyn PdfWithOrigin>::new()
        }
    }

    pub fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
        self.objects.hit(r, ray_t)
    }

    pub fn push(&mut self, object: impl Hit + 'static) {
        self.objects.push(object);
    }

    pub fn push_light(&mut self, light: impl PdfWithOrigin + Hit + 'static) {
        self.lights.push(light);
    }

    pub fn get_lights(&self) -> &HitableList<dyn PdfWithOrigin> {
        &self.lights
    }

    pub fn objects_to_bvh(self) -> Self {
        let bvh = BvhNode::from_list(self.objects);
        let lights = self.lights;
        let mut objects = HitableList::<dyn Hit>::new();
        objects.push(bvh);
        Self {
            objects,
            lights
        }
    }
}