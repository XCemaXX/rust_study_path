use crate::coords::Coords;

#[derive(Default, Clone)]
pub struct Ray {
    orig: Coords,
    dir: Coords,
    tm: f32,
}

impl Ray {
    pub fn new(a: Coords, b: Coords) -> Self {
        Self::new_timed(a, b, 0.)
    }

    pub fn new_timed(a: Coords, b: Coords, time: f32) -> Self {
        Self { orig: a, dir: b, tm: time }
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

    pub fn time(&self) -> f32 {
        self.tm
    }
}
