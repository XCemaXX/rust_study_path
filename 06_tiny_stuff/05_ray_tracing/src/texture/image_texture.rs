use crate::Color;

use super::{Texture, clamp, load_png};

pub struct ImageTexture {
    pixels: Vec<Vec<Color>>,
}

impl ImageTexture {
    pub fn from_png(png_data: &[u8]) -> Self {
        match load_png(png_data) {
            Ok(pixels) => Self { pixels },
            Err(err) => panic!("Error during load png image: {}", err),
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f32, v: f32, _: crate::coords::Coords) -> Color {
        let height = self.pixels.len();
        if height == 0 {
            return Color::new(0.0, 1.0, 1.0);
        }
        let width = self.pixels[0].len();

        let u = clamp(&(0.0..1.0), u);
        let v = 1.0 - clamp(&(0.0..1.0), v);

        let j = (u * width as f32).floor() as usize;
        let i = (v * height as f32).floor() as usize;
        let pixel = self.pixels[i][j];

        pixel
    }
}
