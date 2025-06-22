use std::sync::mpsc::channel;

use image::{ImageBuffer, Rgb};
use num::Complex;
use thread_pool::ThreadPool;

fn normalize(color: f32, factor: f32) -> u8 {
    ((color * factor).powf(0.8) * 255.) as u8
}

fn wavelength_to_rgb(wavelength: u32) -> Rgb<u8> {
    let wave = wavelength as f32;
    let (r, g, b) = match wavelength {
        380..=439 => ((440. - wave) / (440. - 380.), 0.0, 1.0),
        440..=489 => (0.0, (wave - 440.) / (490. - 440.), 1.0),
        490..=509 => (0.0, 1.0, (510. - wave) / (510. - 490.)),
        510..=579 => ((wave - 510.) / (580. - 510.), 1.0, 0.0),
        580..=644 => (1.0, (645. - wave) / (645. - 580.), 0.0),
        645..=780 => (1.0, 0.0, 0.0),
        _ => (0.0, 0.0, 0.0),
    };

    let factor = match wavelength {
        380..=419 => 0.3 + 0.7 * (wave - 380.) / (420. - 380.),
        701..=780 => 0.3 + 0.7 * (780. - wave) / (780. - 700.),
        _ => 1.0,
    };

    let (r, g, b) = (
        normalize(r, factor),
        normalize(g, factor),
        normalize(b, factor),
    );
    [r, g, b].into()
}

fn julia(c: Complex<f32>, x: u32, y: u32, width: u32, height: u32, max_iter: u32) -> u32 {
    let mut z = Complex {
        re: 3.0 * (x as f32 - 0.5 * width as f32) / width as f32,
        im: 2.0 * (y as f32 - 0.5 * height as f32) / height as f32,
    };
    let mut i = 0;
    for t in 0..max_iter {
        if z.norm() >= 2.0 {
            break;
        }
        z = z * z + c;
        i = t;
    }
    i
}

fn main() {
    let (pool_send, pool) = ThreadPool::fixed_size(num_cpus::get());
    let (sender, recv) = channel();

    let (width, height) = (2000, 1000);
    let mut img = ImageBuffer::new(width, height);
    let iterations = 500;

    let c = Complex::new(-0.8, 0.156);

    for y in 0..height {
        let sender = sender.clone();
        pool_send
            .send(move || {
                for x in 0..width {
                    let i = julia(c, x, y, width, height, iterations);
                    let pixel = wavelength_to_rgb(380 + i * 400 / iterations);
                    sender.send((x, y, pixel)).unwrap();
                }
            })
            .unwrap();
    }

    for _ in 0..(width * height) {
        let (x, y, pixel) = recv.recv().expect("Fail to receive all pixels");
        img.put_pixel(x, y, pixel);
    }
    img.save("12_cookbook/04_concurrency/julia.png")
        .expect("Cannot save image");
    pool.shutdown();
}
