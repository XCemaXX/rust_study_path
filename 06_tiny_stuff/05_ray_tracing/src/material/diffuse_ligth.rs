use std::sync::Arc;

use crate::{
    Color,
    texture::{SolidColor, Texture},
};

use super::Material;

pub struct DiffuseLight {
    texture: Arc<Box<dyn Texture>>,
}

impl DiffuseLight {
    pub fn from_color(emit: Color) -> Self {
        Self {
            texture: Arc::new(Box::new(SolidColor::new(emit))),
        }
    }

    pub fn from_shared_texture(texture: Arc<Box<dyn Texture>>) -> Self {
        Self { texture }
    }

    pub fn from_texture(texture: Box<dyn Texture>) -> Self {
        Self {
            texture: Arc::new(texture),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f32, v: f32, p: crate::coords::Coords) -> crate::Color {
        self.texture.value(u, v, p)
    }
}
