mod checker_texture;
mod image_texture;
mod noise_texture;
mod solid_color;
mod texture_loader;

use std::sync::Arc;

use crate::Color;
use crate::coords::Coords;

pub use checker_texture::CheckerTexture;
pub use image_texture::ImageTexture;
pub use noise_texture::NoiseTexture;
pub use solid_color::SolidColor;
pub use texture_loader::clamp;
use texture_loader::load_png;

pub trait Texture: Send + Sync {
    fn value(&self, u: f32, v: f32, p: Coords) -> Color;
}

pub trait IntoSharedTexture {
    fn into_arc(self) -> Arc<dyn Texture>;
}

impl IntoSharedTexture for Arc<dyn Texture> {
    fn into_arc(self) -> Arc<dyn Texture> {
        self
    }
}

impl<T: Texture + 'static> IntoSharedTexture for T {
    fn into_arc(self) -> Arc<dyn Texture> {
        Arc::new(self)
    }
}