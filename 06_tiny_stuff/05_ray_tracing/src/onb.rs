use crate::coords::Coords;


#[derive(Debug, Default, Clone, Copy)]
pub struct Onb {
    axis: [Coords; 3],
}

impl Onb {
    pub fn new(n: Coords) -> Self {
        let w = Coords::unit_vector(n);
        let a = if f32::abs(w.x()) > 0.9 {
            Coords::new(0., 1., 0.)
        } else {
            Coords::new(1., 0., 0.)
        };
        let v = Coords::unit_vector(w.cross(a));
        let u = w.cross(v);
        Self {
            axis: [u, v, w]
        }
    }

    pub fn transform(&self, v: Coords) -> Coords {
        (v.x() * self.u()) + (v.y() * self.v()) + (v.z() * self.w())
    }

    pub fn u(&self) -> Coords {
        self.axis[0]
    }
    pub fn v(&self) -> Coords {
        self.axis[1]
    }
    pub fn w(&self) -> Coords {
        self.axis[2]
    }
}
