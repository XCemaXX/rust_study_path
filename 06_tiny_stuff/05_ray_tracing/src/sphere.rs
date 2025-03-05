use crate::hit::HitRecord;
use crate::material::Material;
use crate::Coords;
use crate::hit;

#[derive(Default)]
pub struct Sphere<T: Material> {
    center: Coords,
    radius: f32,
    material: T,
}

impl<T: Material> Sphere<T> {
    pub fn new(center: Coords, radius: f32, material: T) -> Self {
        Self { center, radius, material }
    }
}

impl<T: Material> hit::Hit for Sphere<T> {
    fn hit(&self, r: &crate::ray::Ray, tmin: f32, tmax: f32) -> Option<hit::HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().dot(r.direction());
        let b = oc.dot(r.direction());
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let sqrt_disc = discriminant.sqrt();
            for t in [(-b - sqrt_disc) / a, (-b + sqrt_disc) / a] {
                if t < tmax && t > tmin {
                    let p = r.point_at_parameter(t);
                    let normal = (p - self.center) / self.radius;
                    return Some(
                        HitRecord{ t, p, normal, material: &self.material });
                }
            }
        }
        None
    }
}