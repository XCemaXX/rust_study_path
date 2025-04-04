use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::pdf::PdfWithOrigin;
use crate::Coords;
use crate::hit::{self, Aabb, Hit, HitRecord};
use crate::material::{IntoSharedMaterial, Material};
use crate::onb::Onb;
use crate::ray::Ray;

use std::f32::consts::PI;

thread_local! {
    static SPHERE_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

#[derive(Clone)]
pub struct Sphere {
    center: Ray,
    radius: f32,
    material: Arc<dyn Material>,
    bbox: Aabb,
    is_moving: bool
}

impl Sphere {
    pub fn new<M: IntoSharedMaterial>(static_center: Coords, radius: f32, material: M) -> Self {
        let material = material.into_arc();
        let radius = radius.max(0.);
        let rvec = Coords::new(radius, radius, radius);
        Self {
            center: Ray::new(static_center, Coords::new(0., 0., 0.)),
            radius,
            material,
            bbox: Aabb::from_points(static_center - rvec, static_center + rvec),
            is_moving: false
        }
    }

    pub fn new_moving<M: IntoSharedMaterial>(
        center1: Coords,
        center2: Coords,
        radius: f32,
        material: M,
    ) -> Self {
        let material = material.into_arc();
        let radius = radius.max(0.);
        let center = Ray::new(center1, center2 - center1);
        let rvec = Coords::new(radius, radius, radius);
        let box1 = Aabb::from_points(center.at(0.) - rvec, center.at(0.) + rvec);
        let box2 = Aabb::from_points(center.at(1.) - rvec, center.at(1.) + rvec);
        Self {
            center,
            radius,
            material,
            bbox: Aabb::from_boxes(box1, box2),
            is_moving: true
        }
    }

    fn get_sphere_uv(p: Coords) -> (f32, f32) {
        // p: a given point on the sphere of radius one, centered at the origin.
        // u: returned value [0,1] of angle around the Y axis from X=-1.
        // v: returned value [0,1] of angle from Y=-1 to Y=+1.
        //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
        //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
        //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>

        let theta = f32::acos(-p.y());
        let phi = f32::atan2(-p.z(), p.x()) + PI;

        let u = phi / (2. * PI);
        let v = theta / PI;
        (u, v)
    }
}

impl hit::Hit for Sphere {
    fn hit(&self, r: &crate::ray::Ray, ray_t: Range<f32>) -> Option<hit::HitRecord> {
        let current_center = self.center.at(r.time());
        let oc = r.origin() - current_center;
        let a = r.direction().dot(r.direction());
        let h = oc.dot(r.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant > 0. {
            let sqrtd = discriminant.sqrt();
            for t in [(-h - sqrtd) / a, (-h + sqrtd) / a] {
                if ray_t.contains(&t) {
                    let p = r.at(t);
                    let outward_normal = (p - current_center) / self.radius;
                    let (u, v) = Self::get_sphere_uv(outward_normal);
                    let mut result = HitRecord::new(t, p, outward_normal, self.material.as_ref())
                        .set_face_normal(r, outward_normal);
                    result.u = u;
                    result.v = v;
                    return Some(result);
                }
            }
        }
        None
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

impl PdfWithOrigin for Sphere {
    fn pdf_value(&self, origin: Coords, direction: Coords) -> f32 {
        assert!(!self.is_moving);
        let ray = Ray::new(origin, direction);
        let Some(_) = self.hit(&ray, 0.001..f32::MAX) else {
            return 0.0;
        };

        let dist_squared = (self.center.at(0.) - origin).length_squared();
        let cos_theta_max = f32::sqrt(1. - self.radius * self.radius / dist_squared);
        let solid_angle = 2. * PI * (1. - cos_theta_max);

        1. / solid_angle
    }

    fn random(&self, origin: Coords) -> Coords {
        let direction = self.center.at(0.) - origin;
        let distance_squared = direction.length_squared();
        let uvw = Onb::new(direction);
        uvw.transform(random_to_sphere(self.radius, distance_squared))
    }
}

fn random_to_sphere(radius: f32, distance_squared: f32) -> Coords {
    let (r1, r2) = SPHERE_RNG.with(|rng| {
        let r1 = rng.borrow_mut().random::<f32>();
        let r2 = rng.borrow_mut().random::<f32>();
        (r1, r2)
    });
    let z = 1. + r2 * (f32::sqrt(1. - radius * radius / distance_squared) - 1.);
    let phi = 2. * PI * r1;
    let x = f32::cos(phi) * f32::sqrt(1. - z * z);
    let y = f32::sin(phi) * f32::sqrt(1. - z * z);

    return Coords::new(x, y, z);
}
