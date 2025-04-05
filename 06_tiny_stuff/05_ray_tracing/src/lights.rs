use std::cell::RefCell;

use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{coords::Coords, pdf::PdfWithOrigin};

thread_local! {
    static LIGHTS_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub struct Lights {
    objects: Vec<Box<dyn PdfWithOrigin>>,
}

impl Lights {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.len() == 0
    }
}

impl Lights {
    pub fn push(&mut self, object: impl PdfWithOrigin + 'static) {
        self.objects.push(Box::new(object));
    }
}

impl PdfWithOrigin for Lights {
    fn pdf_value(&self, origin: Coords, direction: Coords) -> f32 {
        let weight = 1.0 / self.objects.len() as f32;
        let sum = self
            .objects
            .iter()
            .fold(0., |acc, o| acc + weight * o.pdf_value(origin, direction));
        sum
    }

    fn random(&self, origin: Coords) -> Coords {
        assert!(self.objects.len() > 0);
        LIGHTS_RNG.with(|rng| {
            let i = rng.borrow_mut().random_range(0..self.objects.len());
            self.objects[i].random(origin)
        })
    }
}
