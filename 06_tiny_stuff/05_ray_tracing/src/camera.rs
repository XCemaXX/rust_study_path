use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    Coords, Ray,
    color::Color,
    hit::{Hit, HitableList},
};

use std::{cell::RefCell, f32::consts::PI, ops::Range};

pub struct Builder {
    aspect_ratio: f32,
    image_width: usize,
    samples_per_pixel: usize,
    max_depth: usize,
    vfov: f32,
    lookfrom: Coords,
    lookat: Coords,
    vup: Coords,
    defocus_angle: f32,
    focus_dist: f32,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Coords::new(0.0, 0.0, 0.0),
            lookat: Coords::new(0.0, 0.0, -1.0),
            vup: Coords::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
        }
    }
    pub fn aspect_ratio(mut self, x: f32) -> Self {
        self.aspect_ratio = x;
        self
    }
    pub fn image_width(mut self, x: usize) -> Self {
        self.image_width = x;
        self
    }
    pub fn samples_per_pixel(mut self, x: usize) -> Self {
        self.samples_per_pixel = x;
        self
    }
    pub fn max_depth(mut self, x: usize) -> Self {
        self.max_depth = x;
        self
    }
    pub fn vfov(mut self, x: f32) -> Self {
        self.vfov = x;
        self
    }
    pub fn lookfrom(mut self, x: Coords) -> Self {
        self.lookfrom = x;
        self
    }
    pub fn lookat(mut self, x: Coords) -> Self {
        self.lookat = x;
        self
    }
    pub fn vup(mut self, x: Coords) -> Self {
        self.vup = x;
        self
    }
    pub fn defocus_angle(mut self, x: f32) -> Self {
        self.defocus_angle = x;
        self
    }
    pub fn focus_dist(mut self, x: f32) -> Self {
        self.focus_dist = x;
        self
    }

    pub fn build(self) -> Camera {
        let image_height = self.image_width as f32 / self.aspect_ratio;
        let image_height = if image_height < 1.0 {
            1
        } else {
            image_height as usize
        };
        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f32;
        let center = self.lookfrom;
        let theta = degrees_to_radians(self.vfov);
        let h = f32::tan(theta / 2.0);
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f32 / image_height as f32);
        let w = (self.lookfrom - self.lookat).unit_vector();
        let u = self.vup.cross(w).unit_vector();
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = -viewport_height * v;
        let pixel_delta_u = viewport_u / self.image_width as f32;
        let pixel_delta_v = viewport_v / image_height as f32;

        let viewport_upper_left =
            center - (self.focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius =
            self.focus_dist * f32::tan(degrees_to_radians(self.defocus_angle / 2.0));
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;
        Camera::new(
            pixel_samples_scale,
            pixel_delta_u,
            pixel_delta_v,
            pixel00_loc,
            center,
            Image {
                width: self.image_width,
                height: image_height,
            },
            self.samples_per_pixel,
            self.max_depth,
            defocus_disk_u,
            defocus_disk_v,
            self.defocus_angle,
        )
    }
}

pub struct Image {
    pub width: usize,
    pub height: usize,
}

pub struct Camera {
    pixel_samples_scale: f32, // Color scale factor for a sum of pixel samples
    pixel_delta_u: Coords,    // Offset to pixel to the right
    pixel_delta_v: Coords,    // Offset to pixel below
    pixel00_loc: Coords,      // Location of pixel 0, 0
    center: Coords,           // Camera center
    pub image: Image,
    samples_per_pixel: usize,
    max_depth: usize,
    defocus_disk_u: Coords, // Defocus disk horizontal radius
    defocus_disk_v: Coords, // Defocus disk vertical radius

    defocus_angle: f32, // Variation angle of rays through each pixel

    rng: RefCell<SmallRng>,
}

fn random_in_unit_disk(rng: &mut impl Rng) -> Coords {
    loop {
        let p = Coords::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            0.0,
        );
        if p.length_squared() < 1.0 {
            break p;
        }
    }
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

fn clamp(range: &Range<f32>, x: f32) -> f32 {
    if x < range.start {
        range.start
    } else if x > range.end {
        range.end
    } else {
        x
    }
}

impl Camera {
    fn defocus_disk_sample(&self) -> Coords {
        let p = random_in_unit_disk(&mut self.rng.borrow_mut());
        return self.center + (p.x() * self.defocus_disk_u) + (p.y() * self.defocus_disk_v);
    }

    fn sample_square(&self) -> Coords {
        let rng = &mut self.rng.borrow_mut();
        Coords::new(rng.random::<f32>() - 0.5, rng.random::<f32>() - 0.5, 0.0)
    }

    pub fn get_ray(&self, i: usize, j: usize) -> Ray {
        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
            + ((i as f32 + offset.x()) * self.pixel_delta_u)
            + ((j as f32 + offset.y()) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }

    fn color_to_8b_format(pixel_color: Color) -> Color {
        let mut pixel_color = pixel_color;
        pixel_color = pixel_color.sqrt_axis();
        let intensity = &(0.0..0.999);
        let color = Color::new(
            256.0 * clamp(intensity, pixel_color.r()),
            256.0 * clamp(intensity, pixel_color.g()),
            256.0 * clamp(intensity, pixel_color.b()),
        );
        color
    }

    pub fn render(&self, world: HitableList) -> Vec<Color> {
        let mut result = Vec::with_capacity(self.image.height * self.image.width);
        for j in 0..self.image.height {
            for i in 0..self.image.width {
                let mut pixel_color = Color::default();
                for _ in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += self.ray_color(r, &world, self.max_depth);
                }
                let color = Self::color_to_8b_format(self.pixel_samples_scale * pixel_color);
                result.push(color);
            }
        }
        result
    }

    fn ray_color(&self, r: Ray, world: &HitableList, depth: usize) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(rec) = world.hit(&r, 0.001..f32::MAX) {
            if let Some((scattered, attenuation)) = rec.material.scatter(&r, &rec) {
                attenuation * self.ray_color(scattered, world, depth - 1)
            } else {
                Color::new(0.0, 0.0, 0.0)
            }
        } else {
            let unit_direction = r.direction().unit_vector();
            let a = 0.5 * (unit_direction.y() + 1.0);
            (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
        }
    }

    fn new(
        pixel_samples_scale: f32,
        pixel_delta_u: Coords,
        pixel_delta_v: Coords,
        pixel00_loc: Coords,
        center: Coords,
        image: Image,
        samples_per_pixel: usize,
        max_depth: usize,
        defocus_disk_u: Coords,
        defocus_disk_v: Coords,
        defocus_angle: f32,
    ) -> Self {
        Self {
            pixel_samples_scale,
            pixel_delta_u,
            pixel_delta_v,
            pixel00_loc,
            center,
            image,
            samples_per_pixel,
            max_depth,
            defocus_disk_u,
            defocus_disk_v,
            defocus_angle,
            rng: RefCell::new(SmallRng::from_rng(&mut rand::rng())),
        }
    }
}
