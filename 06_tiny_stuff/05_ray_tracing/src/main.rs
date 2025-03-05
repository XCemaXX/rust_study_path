mod vec3;
mod color;
mod coords;
mod ray;
mod sphere;
mod hit;
mod camera;
mod material;

use camera::Camera;
use color::Color;
use coords::Coords;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use ray::Ray;
use hit::{Hit, HitableList};
use sphere::Sphere;
use vec3::Vec3;
use material::{Albedo, Dielectric, Lambertian, Metal};

fn color(r: Ray, world: &HitableList, depth: i32) -> Color {
    if let Some(rec) = world.hit(&r, 0.001, f32::MAX) {
        if depth < 50 {
            if let Some((scattered, attenuation)) = rec.material.scatter(&r, &rec) {
                return attenuation * color(scattered, world, depth + 1);
            }
        }
        Color::new(0.0, 0.0, 0.0)
    } else {
        let unit_direction = r.direction().unit_vector();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn _simple_scene() -> HitableList {
    let mut world = HitableList::new();
    world.push(Box::new(
        Sphere::new(
            Vec3::new(0.0, 0.0, -1.2), 0.5, 
            Lambertian::new(Albedo::new(0.1, 0.2, 0.5)))
    ));
    let ground = Lambertian::new(Albedo::new(0.8, 0.8, 0.0));
    world.push(Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, ground)));
    world.push(Box::new(
        Sphere::new(
            Vec3::new(1.0, 0.0, -1.0), 0.5, 
            Metal::new(Albedo::new(0.8, 0.6, 0.2), 0.1))
    ));
    world.push(Box::new(
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0), 0.5, 
            Dielectric::new(1.5))
    ));
    world.push(Box::new(
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0), 0.4, 
            Dielectric::new(1.0 / 1.5))
    ));
    world
}

fn random_scene() -> HitableList {
    let mut rng = SmallRng::from_rng(&mut rand::rng());
    let mut world = HitableList::new();
    let ground = Lambertian::new(Albedo::new(0.5, 0.5, 0.5));
    world.push(Box::new(Sphere::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0,ground)));

    for a in -11..11 {
        for b in -11..11 {
            let center = Coords::new(
                a as f32 + 0.9 * rng.random::<f32>(), 
                0.2, 
                b as f32 + 0.9 * rng.random::<f32>());
            if (center - Coords::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let choose_mat = rng.random::<f32>();
                if choose_mat < 0.8 {
                    let diffuse = Lambertian::new(Albedo::new(
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>()));
                    world.push(Box::new(Sphere::new(center, 0.2, diffuse)));
                } else if choose_mat < 0.95 {
                    let metal = Metal::new(Albedo::new(
                        0.5 * (1.0 + rng.random::<f32>()),
                        0.5 * (1.0 + rng.random::<f32>()),
                        0.5 * (1.0 + rng.random::<f32>())),
                    0.5 * rng.random::<f32>());
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
    world.push(Box::new(Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0, diffuse)));
    let metal = Metal::new(Albedo::new(0.7, 0.6, 0.5), 0.0);
    world.push(Box::new(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, metal)));
    world
}

fn main() {
    let mut rng = SmallRng::from_rng(&mut rand::rng()); //rand::rng();
    let nx = 200; // 800
    let ny = 100; // 400
    let ns = 100;
    println!("P3\n{nx} {ny}\n255");
    let world = random_scene();

    let lookfrom = Coords::new(13.0, 2.0, 3.0);
    let lookat = Coords::new(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0; //(lookfrom - lookat).length();
    let aperture = 0.3;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Coords::new(0.0, 1.0, 0.0),
        20.0, 
        nx as f32 / ny as f32,
        aperture, dist_to_focus,
    );

    for j in (0..ny).rev() {
        for i in 0..nx {
            let mut c = Color::default();
            for _ in 0..ns {
                let u = (i as f32 + rng.random::<f32>()) / nx as f32;
                let v = (j as f32 + rng.random::<f32>()) / ny as f32;
                let r = camera.get_ray(u, v);

                c += color(r, &world, 0);
            }
            c /= ns as f32;
            c = c.sqrt_axis();
            c *= 255.99;
            println!("{}", c.to_string());
        }
    }
}
