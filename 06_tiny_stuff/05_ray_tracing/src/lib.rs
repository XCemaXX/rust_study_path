mod camera;
mod color;
mod coords;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec3;

pub use color::Color;
use coords::Coords;
use hit::HitableList;
use material::{Albedo, Dielectric, Lambertian, Metal};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use ray::Ray;
use sphere::Sphere;
use vec3::Vec3;

pub fn simple_scene() -> HitableList {
    let mut world = HitableList::new();
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.2),
        0.5,
        Lambertian::new(Albedo::new(0.1, 0.2, 0.5)),
    )));
    let ground = Lambertian::new(Albedo::new(0.8, 0.8, 0.0));
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        ground,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        Metal::new(Albedo::new(0.8, 0.6, 0.2), 0.1),
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

pub fn random_scene() -> HitableList {
    let mut rng = SmallRng::from_rng(&mut rand::rng());
    let mut world = HitableList::new();
    let ground = Lambertian::new(Albedo::new(0.5, 0.5, 0.5));
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
                        Lambertian::new(Albedo::random1(&mut rng) * Albedo::random1(&mut rng));
                    world.push(Box::new(Sphere::new(center, 0.2, diffuse)));
                } else if choose_mat < 0.93 {
                    let metal = Metal::new(
                        Albedo::random(&mut rng, 0.5..1.0),
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
    let diffuse = Lambertian::new(Albedo::new(0.4, 0.2, 0.1));
    world.push(Box::new(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        diffuse,
    )));
    let metal = Metal::new(Albedo::new(0.7, 0.6, 0.5), 0.0);
    world.push(Box::new(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, metal)));
    world
}

pub struct Image {
    pub pixels: Vec<Color>,
    pub width: u32,
    pub height: u32,
}

pub fn render_world() -> Image {
    //let world = simple_scene();
    let world = random_scene();

    let camera = camera::Builder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(200)
        //.image_width(800)
        .samples_per_pixel(100)
        .max_depth(50)
        .vfov(20.0)
        .lookfrom(Coords::new(13.0, 2.0, 3.0))
        .lookat(Coords::new(0.0, 0.0, 0.0))
        .vup(Coords::new(0.0, 1.0, 0.0))
        .defocus_angle(0.6)
        .focus_dist(10.0)
        .build();
    let pixels = camera.render(world);

    Image {
        pixels: pixels,
        width: camera.image.width as u32,
        height: camera.image.height as u32,
    }
}
