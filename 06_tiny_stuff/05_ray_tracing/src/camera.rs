use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{Coords, Ray};

use std::{cell::RefCell, f32::consts::PI};

pub struct Camera {
    origin: Coords, 
    lower_left_corner: Coords,
    horizontal: Coords,
    vertical: Coords,
    lens_radius: f32,
    u: Coords,
    v: Coords,
    w: Coords,
    rng: RefCell<SmallRng>,
}

fn random_in_unit_disk(rng: &mut SmallRng) -> Coords {
    loop {
        let p = 2.0
            * Coords::new(rng.random::<f32>(), rng.random::<f32>(), 0.0) 
            - Coords::new(1.0, 1.0, 0.0);
        break p;
    }
}

impl Camera {


    pub fn new(lookfrom: Coords, lookat: Coords, vup: Coords, vfov: f32, aspect: f32,
        aperture: f32, focus_dist: f32) -> Self {
        let theta = vfov * PI / 180.0;
        let half_height = f32::tan(theta / 2.0);
        let half_width = aspect * half_height;
        let w = (lookfrom - lookat).unit_vector();
        let u = vup.cross(w).unit_vector();
        let v = w.cross(u);
        Self {
            lens_radius: aperture / 2.0,
            lower_left_corner: lookfrom - half_width * focus_dist * u - half_height * focus_dist * v - focus_dist * w,
            horizontal: 2.0 * half_width * focus_dist * u,
            vertical: 2.0 * half_height * focus_dist * v,
            origin: lookfrom,
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng())),
            w, u, v
        }
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk(&mut self.rng.borrow_mut());
        let offset = self.u * rd.x() + self.v * rd.y();
        let b = self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset;
        Ray::new(self.origin + offset, b)
    }

}