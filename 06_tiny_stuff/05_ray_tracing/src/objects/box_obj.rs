use crate::{
    coords::Coords,
    hit::{Aabb, Hit, HitRecord, HitableList},
    material::IntoSharedMaterial,
};

use super::Quad;

pub struct BoxObj {
    sides: HitableList<dyn Hit>,
}

impl BoxObj {
    #[rustfmt::skip]
    pub fn new<M: IntoSharedMaterial>(a: Coords, b: Coords, material: M) -> Self {
        let material = material.into_arc();
        let mut sides = HitableList::<dyn Hit>::new();
        let min = Coords::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z()));
        let max = Coords::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z()));

        let dx = Coords::new(max.x() - min.x(), 0., 0.);
        let dy = Coords::new(0., max.y() - min.y(), 0.);
        let dz = Coords::new(0., 0., max.z() - min.z());

        sides.push(Quad::new(Coords::new(min.x(), min.y(), max.z()), dx, dy, material.clone()));
        sides.push(Quad::new(Coords::new(max.x(), min.y(), max.z()), -dz,  dy, material.clone()));
        sides.push(Quad::new(Coords::new(max.x(), min.y(), min.z()), -dx,  dy, material.clone()));
        sides.push(Quad::new(Coords::new(min.x(), min.y(), min.z()),  dz,  dy, material.clone()));
        sides.push(Quad::new(Coords::new(min.x(), max.y(), max.z()),  dx, -dz, material.clone()));
        sides.push(Quad::new(Coords::new(min.x(), min.y(), min.z()),  dx,  dz, material));

        Self { sides }
    }
}

impl Hit for BoxObj {
    fn hit(&self, r: &crate::ray::Ray, ray_t: std::ops::Range<f32>) -> Option<HitRecord> {
        self.sides.hit(r, ray_t)
    }

    fn bounding_box(&self) -> &Aabb {
        self.sides.bounding_box()
    }
}
