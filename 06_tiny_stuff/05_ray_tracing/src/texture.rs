mod checker_texture;
mod solid_color;
use crate::Color;
use crate::coords::Coords;

pub use checker_texture::CheckerTexture;
pub use solid_color::SolidColor;

pub trait Texture {
    fn value(&self, u: f32, v: f32, p: Coords) -> Color;
}
