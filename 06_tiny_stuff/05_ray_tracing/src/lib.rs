mod camera;
mod color;
mod coords;
mod hit;
mod material;
mod objects;
mod ray;
mod texture;
mod vec3;

use std::sync::Arc;

use camera::Camera;
pub use color::Color;
use coords::Coords;
use hit::{BvhNode, HitableList};
use material::{Dielectric, Lambertian, Metal};
use objects::{Quad, Sphere};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use ray::Ray;
use texture::{CheckerTexture, ImageTexture, NoiseTexture, Texture};
use vec3::Vec3;

fn camera_one() -> Camera {
    Camera::builder()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        //.image_width(800)
        .samples_per_pixel(100)
        .max_depth(50)
        .vfov(20.0)
        .lookfrom(Coords::new(13.0, 2.0, 3.0))
        .lookat(Coords::new(0.0, 0.0, 0.0))
        .vup(Coords::new(0.0, 1.0, 0.0))
        .defocus_angle(0.6)
        .focus_dist(10.0)
        .build()
}

pub fn simple_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.2),
        0.5,
        Lambertian::from_color(Color::new(0.1, 0.2, 0.5)),
    )));
    let ground = Lambertian::from_color(Color::new(0.8, 0.8, 0.0));
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        ground,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        Metal::new(Color::new(0.8, 0.6, 0.2), 0.1),
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        Dielectric::new(1.5),
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.4,
        Dielectric::new(1.0 / 1.5),
    )));
    (world, camera_one())
}

pub fn bouncing_spheres_scene() -> (HitableList, Camera) {
    let mut rng = SmallRng::from_rng(&mut rand::rng());
    let mut world = HitableList::new();

    let checker = Box::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    let ground = Lambertian::from_texture(checker);
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let center = Coords::new(
                a as f32 + 0.9 * rng.random::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.random::<f32>(),
            );
            if (center - Coords::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let choose_mat = rng.random::<f32>();
                if choose_mat < 0.75 {
                    let diffuse =
                        Lambertian::from_color(Color::random1(&mut rng) * Color::random1(&mut rng));
                    let center2 = center + Coords::new(0.0, rng.random_range(0.0..0.5), 0.0);
                    world.push(Box::new(Sphere::new_moving(center, center2, 0.2, diffuse)));
                } else if choose_mat < 0.93 {
                    let metal = Metal::new(
                        Color::random(&mut rng, 0.5..1.0),
                        rng.random_range(0.0..0.5),
                    );
                    world.push(Box::new(Sphere::new(center, 0.2, metal)));
                } else {
                    let glass = Dielectric::new(1.5);
                    world.push(Box::new(Sphere::new(center, 0.2, glass)));
                }
            }
        }
    }
    let glass = Dielectric::new(1.5);
    world.push(Box::new(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, glass)));
    let diffuse = Lambertian::from_color(Color::new(0.4, 0.2, 0.1));
    world.push(Box::new(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        diffuse,
    )));
    let metal = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);
    world.push(Box::new(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, metal)));
    (world, camera_one())
}

pub fn checkered_spheres_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();

    let checker: Box<dyn Texture> = Box::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    let checker = Arc::new(checker);
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -10.0, 0.0),
        10.0,
        Lambertian::from_shared_texture(checker.clone()),
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, 10.0, 0.0),
        10.0,
        Lambertian::from_shared_texture(checker),
    )));
    (world, camera_one())
}

pub fn earth_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    const EARTH_TEXTURE_RAW: &'static [u8] = include_bytes!("../assets/earthmap.png");
    let earth = Lambertian::from_texture(Box::new(ImageTexture::from_png(EARTH_TEXTURE_RAW)));
    let globe = Box::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0, earth));
    world.push(globe);
    (world, camera_one())
}

pub fn perlin_spheres_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let noise: Box<dyn Texture> = Box::new(NoiseTexture::new(4.0));
    let noise = Arc::new(noise);
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Lambertian::from_shared_texture(noise.clone()),
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Lambertian::from_shared_texture(noise),
    )));

    (world, camera_one())
}

fn quads_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let left_red = Lambertian::from_color(Color::new(1.0, 0.2, 0.2));
    let back_green = Lambertian::from_color(Color::new(0.2, 1.0, 0.2));
    let right_blue = Lambertian::from_color(Color::new(0.2, 0.2, 1.0));
    let upper_orange = Lambertian::from_color(Color::new(1.0, 0.5, 0.0));
    let lower_teal = Lambertian::from_color(Color::new(0.2, 0.8, 0.8));

    world.push(Box::new(Quad::new(
        Coords::new(-3.0, -2.0, 5.0),
        Coords::new(0.0, 0.0, -4.0),
        Coords::new(0.0, 4.0, 0.0),
        left_red,
    )));
    world.push(Box::new(Quad::new(
        Coords::new(-2.0, -2.0, 0.0),
        Coords::new(4.0, 0.0, 0.0),
        Coords::new(0.0, 4.0, 0.0),
        back_green,
    )));
    world.push(Box::new(Quad::new(
        Coords::new(3.0, -2.0, 1.0),
        Coords::new(0.0, 0.0, 4.0),
        Coords::new(0.0, 4.0, 0.0),
        right_blue,
    )));
    world.push(Box::new(Quad::new(
        Coords::new(-2.0, 3.0, 1.0),
        Coords::new(4.0, 0.0, 0.0),
        Coords::new(0.0, 0.0, 4.0),
        upper_orange,
    )));
    world.push(Box::new(Quad::new(
        Coords::new(-2.0, -3.0, 5.0),
        Coords::new(4.0, 0.0, 0.0),
        Coords::new(0.0, 0.0, -4.0),
        lower_teal,
    )));

    let camera = Camera::builder()
        .aspect_ratio(1.0)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .vfov(80.0)
        .lookfrom(Coords::new(0.0, 0.0, 9.0))
        .lookat(Coords::new(0.0, 0.0, 0.0))
        .vup(Coords::new(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .build();

    (world, camera)
}

pub struct Image {
    pub pixels: Vec<Color>,
    pub width: u32,
    pub height: u32,
}

pub fn render_world() -> Image {
    let i = 100;
    let (world, camera) = match i {
        0 => simple_scene(),
        1 => simple_scene(),
        2 => bouncing_spheres_scene(),
        3 => checkered_spheres_scene(),
        4 => earth_scene(),
        5 => perlin_spheres_scene(),
        _ => quads_scene(),
    };

    let world = BvhNode::from_list(world);

    let pixels = camera.render(&world);

    Image {
        pixels: pixels,
        width: camera.image.width as u32,
        height: camera.image.height as u32,
    }
}
