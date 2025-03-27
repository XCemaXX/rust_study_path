use std::sync::Arc;

use crate::{
    Color,
    texture::{IntoSharedTexture, SolidColor, Texture},
};

use super::Material;

pub struct DiffuseLight {
    texture: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn from_color(emit: Color) -> Self {
        Self {
            texture: Arc::new(SolidColor::new(emit)),
        }
    }

    pub fn from_texture<T: IntoSharedTexture>(texture: T) -> Self {
        Self {
            texture: texture.into_arc(),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f32, v: f32, p: crate::coords::Coords) -> crate::Color {
        self.texture.value(u, v, p)
    }
}
