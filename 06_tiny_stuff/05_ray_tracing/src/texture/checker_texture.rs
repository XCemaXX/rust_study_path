use std::{rc::Rc, sync::Arc};

use crate::Color;

use super::{SolidColor, Texture};

pub struct CheckerTexture {
    inv_scale: f32,
    even: Arc<Box<dyn Texture + Send + Sync>>,
    odd: Arc<Box<dyn Texture + Send + Sync>>,
}

impl CheckerTexture {
    pub fn from_textures(
        scale: f32,
        even: Arc<Box<dyn Texture + Send + Sync>>,
        odd: Arc<Box<dyn Texture + Send + Sync>>,
    ) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }

    pub fn from_colors(scale: f32, c1: Color, c2: Color) -> Self {
        let c1: Box<dyn Texture + Send + Sync> = Box::new(SolidColor::new(c1));
        let c2: Box<dyn Texture + Send + Sync> = Box::new(SolidColor::new(c2));

        Self {
            inv_scale: 1.0 / scale,
            even: Arc::new(c1),
            odd: Arc::new(c2),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f32, v: f32, p: crate::coords::Coords) -> crate::Color {
        let x = (self.inv_scale * p.x()).floor() as i32;
        let y = (self.inv_scale * p.y()).floor() as i32;
        let z = (self.inv_scale * p.z()).floor() as i32;

        let is_even = (x + y + z) % 2 == 0;
        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}
