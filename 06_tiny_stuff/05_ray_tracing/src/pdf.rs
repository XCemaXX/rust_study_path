use std::{cell::RefCell, f32::consts::PI};

use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{coords::Coords, onb::Onb};

thread_local! {
    static PDF_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

pub trait Pdf {
    fn value(&self, direction: Coords) -> f32;
    fn generate(&self) -> Coords;
}

pub trait PdfWithOrigin: Sync {
    fn pdf_value(&self, origin: Coords, direction: Coords) -> f32;
    fn random(&self, origin: Coords) -> Coords;
}

pub struct SpherePdf {}

impl SpherePdf {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pdf for SpherePdf {
    fn value(&self, _direction: Coords) -> f32 {
        1. / (4. * PI)
    }

    fn generate(&self) -> Coords {
        PDF_RNG.with(|rng| Coords::random_unit_vector(&mut rng.borrow_mut()))
    }
}

pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: Coords) -> Self {
        Self { uvw: Onb::new(w) }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: Coords) -> f32 {
        let cosine_theta = direction.unit_vector().dot(self.uvw.w());
        f32::max(0., cosine_theta / PI)
    }

    fn generate(&self) -> Coords {
        let d = PDF_RNG.with(|rng| Coords::random_cosine_direction(&mut rng.borrow_mut()));
        self.uvw.transform(d)
    }
}


pub struct HitablePdf<'a> {
    origin: Coords,
    objects: &'a dyn PdfWithOrigin,
}

impl<'a> HitablePdf<'a> {
    pub fn new(objects: &'a dyn PdfWithOrigin, origin: Coords) -> Self {
        Self { objects, origin }
    }
}

impl<'a> Pdf for HitablePdf<'a> {
    fn value(&self, direction: Coords) -> f32 {
        self.objects.pdf_value(self.origin, direction)
    }

    fn generate(&self) -> Coords {
        self.objects.random(self.origin)
    }
}

pub struct MixturePdf<'a> {
    pdfs: [&'a dyn Pdf; 2],
}

impl<'a> MixturePdf<'a> {
    pub fn new(p0: &'a dyn Pdf, p1: &'a dyn Pdf) -> Self {
        Self { pdfs: [p0, p1] }
    }
}

impl<'a> Pdf for MixturePdf<'a> {
    fn value(&self, direction: Coords) -> f32 {
        self.pdfs
            .iter()
            .fold(0., |acc, p| acc + 0.5 * p.value(direction))
    }

    fn generate(&self) -> Coords {
        PDF_RNG.with(|rng| {
            if rng.borrow_mut().random::<f32>() < 0.5 {
                self.pdfs[0].generate()
            } else {
                self.pdfs[1].generate()
            }
        })
    }
}
