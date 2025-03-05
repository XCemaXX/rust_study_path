use crate::coords::Coords;

#[derive(Default)]
pub struct Ray {
    a: Coords,
    b: Coords,
}

impl Ray {
    pub fn new(a: Coords, b: Coords) -> Self {
        Self {a, b}
    }

    pub fn origin(&self) -> Coords {
        self.a
    }

    pub fn direction(&self) -> Coords {
        self.b
    }

    pub fn point_at_parameter(&self, t: f32) -> Coords {
        self.a + t * self.b
    }
}