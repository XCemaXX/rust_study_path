mod checker_texture;
mod image_texture;
mod solid_color;
mod texture_loader;

use crate::Color;
use crate::coords::Coords;

pub use checker_texture::CheckerTexture;
pub use image_texture::ImageTexture;
pub use solid_color::SolidColor;
pub use texture_loader::clamp;
use texture_loader::load_png;

pub trait Texture: Send + Sync {
    fn value(&self, u: f32, v: f32, p: Coords) -> Color;
}
