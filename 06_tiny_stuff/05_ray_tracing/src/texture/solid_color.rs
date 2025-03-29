use crate::Color;

use super::Texture;

pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self { albedo: color }
    }
}

impl Texture for SolidColor {
    fn value(&self, _: f32, _: f32, _: crate::coords::Coords) -> Color {
        self.albedo
    }
}

impl From<Color> for SolidColor {
    fn from(color: Color) -> Self {
        SolidColor::new(color)
    }
}