mod camera;
mod color;
mod coords;
mod hit;
mod material;
mod ray;
mod sphere;
mod texture;
mod vec3;

use std::sync::Arc;

use camera::Camera;
pub use color::Color;
use coords::Coords;
use hit::{BvhNode, HitableList};
use material::{Dielectric, Lambertian, Metal};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use ray::Ray;
use sphere::Sphere;
use texture::{CheckerTexture, ImageTexture, Texture};
use vec3::Vec3;

pub fn simple_scene() -> HitableList {
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
    world
}

pub fn bouncing_spheres_scene() -> HitableList {
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
    world
}

pub fn checkered_spheres_scene() -> HitableList {
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
    world
}

pub fn earth_scene() -> HitableList {
    let mut world = HitableList::new();
    const EARTH_TEXTURE_RAW: &'static [u8] = include_bytes!("../assets/earthmap.png");
    let earth = Lambertian::from_texture(Box::new(ImageTexture::from_png(EARTH_TEXTURE_RAW)));
    let globe = Box::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0, earth));
    world.push(globe);
    world
}

pub struct Image {
    pub pixels: Vec<Color>,
    pub width: u32,
    pub height: u32,
}

pub fn render_world() -> Image {
    //let world = simple_scene();
    //let world = bouncing_spheres_scene();
    //let world = checkered_spheres_scene();
    let world = earth_scene();
    let world = BvhNode::from_list(world);

    let camera = Camera::builder()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        //.image_width(800)
        .samples_per_pixel(80)
        .max_depth(50)
        .vfov(20.0)
        .lookfrom(Coords::new(13.0, 2.0, 3.0))
        .lookat(Coords::new(0.0, 0.0, 0.0))
        .vup(Coords::new(0.0, 1.0, 0.0))
        .defocus_angle(0.6)
        .focus_dist(10.0)
        .build();
    let pixels = camera.render(&world);

    Image {
        pixels: pixels,
        width: camera.image.width as u32,
        height: camera.image.height as u32,
    }
}
