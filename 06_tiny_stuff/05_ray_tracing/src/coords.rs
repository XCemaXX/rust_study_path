use crate::vec3::Vec3;

pub type Coords = Vec3<CoordsTag>;

#[derive(Debug, Default, Clone, Copy)]
pub struct CoordsTag;

impl Coords {
    pub fn x(&self) -> f32 {
        self.0
    }
    pub fn y(&self) -> f32 {
        self.1
    }
    pub fn z(&self) -> f32 {
        self.2
    }
}
