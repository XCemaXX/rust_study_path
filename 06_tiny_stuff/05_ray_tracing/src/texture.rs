mod checker_texture;
mod solid_color;
mod texture_loader;
mod image_texture;

use std::ops::Range;

use crate::Color;
use crate::coords::Coords;

pub use checker_texture::CheckerTexture;
pub use solid_color::SolidColor;
pub use image_texture::ImageTexture;
use texture_loader::load_png;

pub trait Texture {
    fn value(&self, u: f32, v: f32, p: Coords) -> Color;
}

pub fn clamp(range: &Range<f32>, x: f32) -> f32 {
    if x < range.start {
        range.start
    } else if x > range.end {
        range.end
    } else {
        x
    }
}