use crate::vec3::Vec3;

pub type Color = Vec3<ColorTag>;

#[derive(Default)]
pub struct ColorTag;

impl Color {
    pub fn r(&self) -> f32 {
        self.0
    }
    pub fn g(&self) -> f32 {
        self.1
    }
    pub fn b(&self) -> f32 {
        self.2
    }
}