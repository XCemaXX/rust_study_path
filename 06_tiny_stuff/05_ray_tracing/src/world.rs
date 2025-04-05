use std::ops::Range;

use crate::{hit::{BvhNode, Hit, HitRecord, HitableList}, lights::Lights, pdf::PdfWithOrigin, ray::Ray};


pub struct World {
    objects: HitableList,
    lights: Lights,
}

impl World {
    pub fn new() -> Self {
        Self {
            objects: HitableList::new(),
            lights: Lights::new()
        }
    }

    pub fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<HitRecord> {
        self.objects.hit(r, ray_t)
    }

    pub fn push(&mut self, object: impl Hit + 'static) {
        self.objects.push(object);
    }

    pub fn push_light(&mut self, light: impl PdfWithOrigin + Hit + Clone + 'static) {
        let o = light.clone();
        self.lights.push(light);
        self.objects.push(o);
    }

    pub fn get_lights(&self) -> &Lights {
        &self.lights
    }

    pub fn objects_to_bvh(self) -> Self {
        let bvh = BvhNode::from_list(self.objects);
        let lights = self.lights;
        let mut objects = HitableList::new();
        objects.push(bvh);
        Self {
            objects,
            lights
        }
    }
}