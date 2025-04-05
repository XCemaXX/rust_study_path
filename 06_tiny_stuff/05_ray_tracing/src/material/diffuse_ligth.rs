use std::sync::Arc;

use crate::{
    coords::Coords, hit::HitRecord, ray::Ray, texture::{IntoSharedTexture, SolidColor, Texture}, Color
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

    #[allow(dead_code)]
    pub fn from_texture<T: IntoSharedTexture>(texture: T) -> Self {
        Self {
            texture: texture.into_arc(),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, _r_in: &Ray, rec: &HitRecord, u: f32, v: f32, p: Coords) -> Color {
        if !rec.front_face {
            Color::new(0.,0.,0.)
        } else {
            self.texture.value(u, v, p)
        }
    }
}
