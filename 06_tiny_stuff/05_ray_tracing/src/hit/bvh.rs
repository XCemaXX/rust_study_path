use std::cmp::Ordering;

use crate::vec3::Axis;

use super::{Aabb, Hit, HitableList};

enum Children {
    Alone(Box<dyn Hit>),
    Pair {
        left: Box<dyn Hit>,
        right: Box<dyn Hit>,
    },
}

pub struct BvhNode {
    bbox: Aabb,
    children: Children,
}

impl BvhNode {
    fn build(objects: Vec<Box<dyn Hit>>) -> Self {
        let mut bbox = Aabb::empty();
        for object in &objects {
            bbox = Aabb::from_boxes(bbox, object.bounding_box());
        }

        let mut objects = objects;
        let axis = bbox.longest_axis();

        if objects.len() == 1 {
            let child = objects.pop().unwrap();
            return Self {
                bbox,
                children: Children::Alone(child),
            };
        }
        let (left, right) = match objects.len() {
            2 => {
                let right = objects.pop().unwrap();
                let left = objects.pop().unwrap();
                (left, right)
            }
            _ => {
                objects.sort_by(|a, b| box_compare(a.as_ref(), b.as_ref(), axis));

                let mid = objects.len() / 2;
                let tail = objects.split_off(mid);
                let left: Box<dyn Hit> = Box::new(Self::build(objects));
                let right: Box<dyn Hit> = Box::new(Self::build(tail));
                (left, right)
            }
        };
        return Self {
            bbox,
            children: Children::Pair { left, right },
        };
    }

    pub fn from_list(list: HitableList) -> Self {
        let objects = list.take_objects();
        Self::build(objects)
    }
}

impl Hit for BvhNode {
    fn hit(
        &self,
        r: &crate::ray::Ray,
        ray_t: std::ops::Range<f32>,
    ) -> Option<crate::hit::HitRecord> {
        self.bbox.hit(r, ray_t.clone())?;

        match &self.children {
            Children::Alone(alone) => alone.hit(r, ray_t),
            Children::Pair { left, right } => {
                let hit_left = left.hit(r, ray_t.clone());
                let max_t = hit_left.as_ref().map_or(ray_t.end, |h| h.t);
                let hit_right = right.hit(r, ray_t.start..max_t);
                hit_right.or(hit_left)
            }
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox.clone()
    }
}

fn box_compare(a: &dyn Hit, b: &dyn Hit, axis: Axis) -> Ordering {
    let a_axis_interval = &a.bounding_box()[axis];
    let b_axis_interval = &b.bounding_box()[axis];
    a_axis_interval
        .start
        .partial_cmp(&b_axis_interval.start)
        .unwrap_or(Ordering::Equal)
}
