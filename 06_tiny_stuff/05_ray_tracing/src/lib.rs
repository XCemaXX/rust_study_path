mod camera;
mod color;
mod coords;
mod hit;
mod material;
mod objects;
mod ray;
mod texture;
mod vec3;

use core::f32;
use std::sync::Arc;

use camera::Camera;
pub use color::Color;
use coords::Coords;
use hit::{BvhNode, Hit, HitableList, Transformable};
use material::{Dielectric, DiffuseLight, Lambertian, Material, Metal};
use objects::{BoxObj, ConstantMedium, Quad, Sphere};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use ray::Ray;
use texture::{CheckerTexture, ImageTexture, NoiseTexture, Texture};
use vec3::Vec3;

const EARTH_TEXTURE_RAW: &'static [u8] = include_bytes!("../assets/earthmap.png");

fn camera_one() -> Camera {
    Camera::builder()
        .aspect_ratio(16. / 9.)
        .image_width(800)
        .samples_per_pixel(100)
        .max_depth(50)
        .vfov(20.)
        .lookfrom(Coords::new(13., 2., 3.))
        .lookat(Coords::new(0., 0., 0.))
        .vup(Coords::new(0., 1., 0.))
        .defocus_angle(0.6)
        .focus_dist(10.)
        .build()
}

fn simple_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    world.push(Sphere::new(
        Vec3::new(0., 0., -1.2),
        0.5,
        Lambertian::from_color(Color::new(0.1, 0.2, 0.5)),
    ));
    let ground = Lambertian::from_color(Color::new(0.8, 0.8, 0.));
    world.push(Sphere::new(Vec3::new(0., -100.5, -1.), 100., ground));
    world.push(Sphere::new(
        Vec3::new(1., 0., -1.),
        0.5,
        Metal::new(Color::new(0.8, 0.6, 0.2), 0.1),
    ));
    world.push(Sphere::new(
        Vec3::new(-1., 0., -1.),
        0.5,
        Dielectric::new(1.5),
    ));
    world.push(Sphere::new(
        Vec3::new(-1., 0., -1.),
        0.4,
        Dielectric::new(1. / 1.5),
    ));
    (world, camera_one())
}

fn bouncing_spheres_scene() -> (HitableList, Camera) {
    let mut rng = SmallRng::from_rng(&mut rand::rng());
    let mut world = HitableList::new();

    let ground = Lambertian::from_texture(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.push(Sphere::new(Vec3::new(0., -1000., 0.), 1000., ground));

    for a in -11..11 {
        for b in -11..11 {
            let center = Coords::new(
                a as f32 + 0.9 * rng.random::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.random::<f32>(),
            );
            if (center - Coords::new(4., 0.2, 0.)).length() > 0.9 {
                let choose_mat = rng.random::<f32>();
                if choose_mat < 0.75 {
                    let diffuse =
                        Lambertian::from_color(Color::random1(&mut rng) * Color::random1(&mut rng));
                    let center2 = center + Coords::new(0., rng.random_range(0.0..0.5), 0.);
                    world.push(Sphere::new_moving(center, center2, 0.2, diffuse));
                } else if choose_mat < 0.93 {
                    let metal =
                        Metal::new(Color::random(&mut rng, 0.5..1.), rng.random_range(0.0..0.5));
                    world.push(Sphere::new(center, 0.2, metal));
                } else {
                    let glass = Dielectric::new(1.5);
                    world.push(Sphere::new(center, 0.2, glass));
                }
            }
        }
    }
    let glass = Dielectric::new(1.5);
    world.push(Sphere::new(Vec3::new(0., 1., 0.), 1., glass));
    let diffuse = Lambertian::from_color(Color::new(0.4, 0.2, 0.1));
    world.push(Sphere::new(Vec3::new(-4., 1., 0.), 1., diffuse));
    let metal = Metal::new(Color::new(0.7, 0.6, 0.5), 0.);
    world.push(Sphere::new(Vec3::new(4., 1., 0.), 1., metal));
    (world, camera_one())
}

fn checkered_spheres_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();

    let checker: Arc<dyn Texture> = Arc::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.push(Sphere::new(
        Vec3::new(0., -10., 0.),
        10.,
        Lambertian::from_texture(checker.clone()),
    ));
    world.push(Sphere::new(
        Vec3::new(0., 10., 0.),
        10.,
        Lambertian::from_texture(checker),
    ));
    (world, camera_one())
}

fn earth_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let earth = Lambertian::from_texture(ImageTexture::from_png(EARTH_TEXTURE_RAW));
    let globe = Sphere::new(Vec3::new(0., 0., 0.), 2., earth);
    world.push(globe);
    (world, camera_one())
}

fn perlin_spheres_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let noise: Arc<dyn Material> = Arc::new(Lambertian::from_texture(NoiseTexture::new(4.)));
    world.push(Sphere::new(Vec3::new(0., -1000., 0.), 1000., noise.clone()));
    world.push(Sphere::new(Vec3::new(0., 2., 0.), 2., noise));

    (world, camera_one())
}

fn quads_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let left_red = Lambertian::from_color(Color::new(1., 0.2, 0.2));
    let back_green = Lambertian::from_color(Color::new(0.2, 1., 0.2));
    let right_blue = Lambertian::from_color(Color::new(0.2, 0.2, 1.));
    let upper_orange = Lambertian::from_color(Color::new(1., 0.5, 0.));
    let lower_teal = Lambertian::from_color(Color::new(0.2, 0.8, 0.8));

    world.push(Quad::new(
        Coords::new(-3., -2., 5.),
        Coords::new(0., 0., -4.),
        Coords::new(0., 4., 0.),
        left_red,
    ));
    world.push(Quad::new(
        Coords::new(-2., -2., 0.),
        Coords::new(4., 0., 0.),
        Coords::new(0., 4., 0.),
        back_green,
    ));
    world.push(Quad::new(
        Coords::new(3., -2., 1.),
        Coords::new(0., 0., 4.),
        Coords::new(0., 4., 0.),
        right_blue,
    ));
    world.push(Quad::new(
        Coords::new(-2., 3., 1.),
        Coords::new(4., 0., 0.),
        Coords::new(0., 0., 4.),
        upper_orange,
    ));
    world.push(Quad::new(
        Coords::new(-2., -3., 5.),
        Coords::new(4., 0., 0.),
        Coords::new(0., 0., -4.),
        lower_teal,
    ));

    let camera = Camera::builder()
        .aspect_ratio(1.)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .vfov(80.)
        .lookfrom(Coords::new(0., 0., 9.))
        .lookat(Coords::new(0., 0., 0.))
        .vup(Coords::new(0., 1., 0.))
        .defocus_angle(0.)
        .build();

    (world, camera)
}

fn simple_light_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let noise: Arc<dyn Material> = Arc::new(Lambertian::from_texture(NoiseTexture::new(4.)));
    world.push(Sphere::new(Vec3::new(0., -1000., 0.), 1000., noise.clone()));
    world.push(Sphere::new(Vec3::new(0., 2., 0.), 2., noise));

    let light = Color::new(4., 4., 4.);
    world.push(Sphere::new(
        Coords::new(0., 7., 0.),
        2.,
        DiffuseLight::from_color(light),
    ));
    world.push(Quad::new(
        Coords::new(3., 1., -2.),
        Coords::new(2., 0., 0.),
        Coords::new(0., 2., 0.),
        DiffuseLight::from_color(light),
    ));

    let camera = Camera::builder()
        .aspect_ratio(16. / 9.)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .background(Color::new(0., 0., 0.))
        .vfov(20.)
        .lookfrom(Coords::new(26., 3., 6.))
        .lookat(Coords::new(0., 2., 0.))
        .vup(Coords::new(0., 1., 0.))
        .defocus_angle(0.)
        .build();

    (world, camera)
}

fn camera_cornel() -> Camera {
    Camera::builder()
        .aspect_ratio(1.)
        .image_width(300)
        .samples_per_pixel(600)
        .max_depth(80)
        .background(Color::new(0., 0., 0.))
        .vfov(40.)
        .lookfrom(Coords::new(278., 278., -800.))
        .lookat(Coords::new(278., 278., 0.))
        .vup(Coords::new(0., 1., 0.))
        .defocus_angle(0.)
        .build()
}

fn cornell_box_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let red = Color::new(0.65, 0.05, 0.05);
    let white: Arc<dyn Material> = Arc::new(Lambertian::from_color(Color::new(0.73, 0.73, 0.73)));
    let green = Lambertian::from_color(Color::new(0.12, 0.45, 0.15));
    let light = Color::new(15., 15., 15.);

    world.push(Quad::new(
        Coords::new(555., 0., 0.),
        Coords::new(0., 555., 0.),
        Coords::new(0., 0., 555.),
        green,
    ));
    world.push(Quad::new(
        Coords::new(0., 0., 0.),
        Coords::new(0., 555., 0.),
        Coords::new(0., 0., 555.),
        Lambertian::from_color(red),
    ));
    world.push(Quad::new(
        Coords::new(343., 554., 332.),
        Coords::new(-130., 0., 0.),
        Coords::new(0., 0., -105.),
        DiffuseLight::from_color(light),
    ));
    world.push(Quad::new(
        Coords::new(0., 0., 0.),
        Coords::new(555., 0., 0.),
        Coords::new(0., 0., 555.),
        white.clone(),
    ));
    world.push(Quad::new(
        Coords::new(555., 555., 555.),
        Coords::new(-555., 0., 0.),
        Coords::new(0., 0., -555.),
        white.clone(),
    ));
    world.push(Quad::new(
        Coords::new(0., 0., 555.),
        Coords::new(555., 0., 0.),
        Coords::new(0., 555., 0.),
        white.clone(),
    ));

    let box1 = BoxObj::new(
        Coords::new(0., 0., 0.),
        Coords::new(165., 330., 165.),
        white.clone(),
    )
    .rotate_y(15.)
    .translate(Coords::new(265., 0., 295.));
    world.push(box1);
    let box2 = BoxObj::new(
        Coords::new(0., 0., 0.),
        Coords::new(165., 165., 165.),
        white.clone(),
    )
    .rotate_y(-18.)
    .translate(Coords::new(130., 0., 65.));
    world.push(box2);

    (world, camera_cornel())
}

fn cornell_smoke_scene() -> (HitableList, Camera) {
    let mut world = HitableList::new();
    let red = Color::new(0.65, 0.05, 0.05);
    let white: Arc<dyn Material> = Arc::new(Lambertian::from_color(Color::new(0.73, 0.73, 0.73)));
    let green = Lambertian::from_color(Color::new(0.12, 0.45, 0.15));
    let light = Color::new(15., 15., 15.);

    world.push(Quad::new(
        Coords::new(555., 0., 0.),
        Coords::new(0., 555., 0.),
        Coords::new(0., 0., 555.),
        green,
    ));
    world.push(Quad::new(
        Coords::new(0., 0., 0.),
        Coords::new(0., 555., 0.),
        Coords::new(0., 0., 555.),
        Lambertian::from_color(red),
    ));
    world.push(Quad::new(
        Coords::new(343., 554., 332.),
        Coords::new(-130., 0., 0.),
        Coords::new(0., 0., -105.),
        DiffuseLight::from_color(light),
    ));
    world.push(Quad::new(
        Coords::new(0., 0., 0.),
        Coords::new(555., 0., 0.),
        Coords::new(0., 0., 555.),
        white.clone(),
    ));
    world.push(Quad::new(
        Coords::new(555., 555., 555.),
        Coords::new(-555., 0., 0.),
        Coords::new(0., 0., -555.),
        white.clone(),
    ));
    world.push(Quad::new(
        Coords::new(0., 0., 555.),
        Coords::new(555., 0., 0.),
        Coords::new(0., 555., 0.),
        white.clone(),
    ));

    let box1 = BoxObj::new(
        Coords::new(0., 0., 0.),
        Coords::new(165., 330., 165.),
        white.clone(),
    )
    .rotate_y(15.)
    .translate(Coords::new(265., 0., 295.));
    let box2 = BoxObj::new(
        Coords::new(0., 0., 0.),
        Coords::new(165., 165., 165.),
        white.clone(),
    )
    .rotate_y(-18.)
    .translate(Coords::new(130., 0., 65.));

    let smoked_black_box = ConstantMedium::from_color(box1, 0.01, Color::new(0., 0., 0.));
    let smoked_noisy_box = ConstantMedium::from_texture(box2, 0.01, NoiseTexture::new(0.2));
    world.push(smoked_black_box);
    world.push(smoked_noisy_box);

    (world, camera_cornel())
}

fn final_scene(
    image_width: usize,
    samples_per_pixel: usize,
    max_depth: usize,
) -> (HitableList, Camera) {
    let mut rng = SmallRng::from_rng(&mut rand::rng());

    let ground: Arc<dyn Material> = Arc::new(Lambertian::from_color(Color::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    let ground_boxes = (0..boxes_per_side)
        .flat_map(|i| (0..boxes_per_side).map(move |j| (i, j)))
        .map(|(i, j)| {
            let w = 100.;
            let a = Coords::new(-1000. + i as f32 * w, 0., -1000. + j as f32 * w);
            let b = Coords::new(a.x() + w, rng.random_range(1.0..101.), a.z() + w);
            Box::new(BoxObj::new(a, b, ground.clone())) as Box<dyn Hit>
        })
        .collect::<HitableList>();

    let mut world = HitableList::new();
    world.push(BvhNode::from_list(ground_boxes));

    let light: DiffuseLight = DiffuseLight::from_color(Color::new(7., 7., 7.));
    world.push(Quad::new(
        Coords::new(123., 554., 147.),
        Coords::new(300., 0., 0.),
        Coords::new(0., 0., 265.),
        light,
    ));

    let center1 = Coords::new(400., 400., 200.);
    world.push(Sphere::new_moving(
        center1,
        center1 + Coords::new(30., 0., 0.),
        50.,
        Lambertian::from_color(Color::new(0.7, 0.3, 0.1)),
    ));

    world.push(Sphere::new(
        Coords::new(260., 150., 45.),
        50.,
        Dielectric::new(1.5),
    ));
    world.push(Sphere::new(
        Coords::new(0., 150., 145.),
        50.,
        Metal::new(Color::new(0.8, 0.8, 0.9), 1.),
    ));

    let boundary = Sphere::new(Coords::new(360., 150., 145.), 70., Dielectric::new(1.5));
    world.push(boundary);
    let boundary = Sphere::new(Coords::new(360., 150., 145.), 70., Dielectric::new(1.5));
    world.push(ConstantMedium::from_color(
        boundary,
        0.2,
        Color::new(0.2, 0.4, 0.9),
    ));
    let boundary = Sphere::new(Coords::new(0., 0., 0.), 5000., Dielectric::new(1.5));
    world.push(ConstantMedium::from_color(
        boundary,
        0.0001,
        Color::new(1., 1., 1.),
    ));

    let emat: Lambertian = Lambertian::from_texture(ImageTexture::from_png(EARTH_TEXTURE_RAW));
    world.push(Sphere::new(Coords::new(400., 200., 400.), 100., emat));
    let permat = Lambertian::from_texture(NoiseTexture::new(0.2));
    world.push(Sphere::new(Coords::new(220., 280., 300.), 80., permat));

    let white: Arc<dyn Material> = Arc::new(Lambertian::from_color(Color::new(0.73, 0.73, 0.73)));
    let ns = 100;
    let boxes2 = (0..ns)
        .map(|_| Sphere::new(Coords::random(&mut rng, 0.0..165.), 10., white.clone()))
        .map(|s| Box::new(s) as Box<dyn Hit>)
        .collect::<HitableList>();

    world.push(
        BvhNode::from_list(boxes2)
            .rotate_y(15.)
            .translate(Coords::new(-100., 270., 395.)),
    );

    let camera = Camera::builder()
        .aspect_ratio(1.)
        .image_width(image_width)
        .samples_per_pixel(samples_per_pixel)
        .max_depth(max_depth)
        .background(Color::new(0., 0., 0.))
        .vfov(40.)
        .lookfrom(Coords::new(478., 478. /*278.*/, -600.))
        .lookat(Coords::new(278., 278., 0.))
        .vup(Coords::new(0., 1., 0.))
        .defocus_angle(0.)
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
        1 => simple_scene(),
        2 => bouncing_spheres_scene(),
        3 => checkered_spheres_scene(),
        4 => earth_scene(),
        5 => perlin_spheres_scene(),
        6 => quads_scene(),
        7 => simple_light_scene(),
        8 => cornell_box_scene(),
        9 => cornell_smoke_scene(),
        10 => final_scene(800, 10000, 40),
        _ => final_scene(400, 250, 4),
    };

    let world = BvhNode::from_list(world);

    let pixels = camera.render(&world);

    Image {
        pixels: pixels,
        width: camera.image.width as u32,
        height: camera.image.height as u32,
    }
}
