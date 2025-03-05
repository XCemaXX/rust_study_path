use crate::coords::Coords;

#[derive(Default)]
pub struct Ray {
    orig: Coords,
    dir: Coords,
}

impl Ray {
    pub fn new(a: Coords, b: Coords) -> Self {
        Self { orig: a, dir: b }
    }

    pub fn origin(&self) -> Coords {
        self.orig
    }

    pub fn direction(&self) -> Coords {
        self.dir
    }

    pub fn at(&self, t: f32) -> Coords {
        self.orig + t * self.dir
    }
}
