use std::cell::RefCell;

use rand::{SeedableRng, rngs::SmallRng, seq::SliceRandom};

use crate::{Color, coords::Coords};

use super::Texture;

pub struct NoiseTexture {
    noise: Perlin,
    scale: f32,
}

impl NoiseTexture {
    pub fn new(scale: f32) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _: f32, _: f32, p: Coords) -> crate::Color {
        let color = Color::new(0.5, 0.5, 0.5)
            * (1. + f32::sin(self.scale * p.z() + 10. * self.noise.turb(p, 7)));
        assert!(color.r() >= 0.);
        assert!(color.g() >= 0.);
        assert!(color.b() >= 0.);
        color
    }
}

const POINT_COUNT: usize = 256;
const OFFSETS: [(i32, i32, i32); 8] = [
    (0, 0, 0),
    (0, 0, 1),
    (0, 1, 0),
    (0, 1, 1),
    (1, 0, 0),
    (1, 0, 1),
    (1, 1, 0),
    (1, 1, 1),
];

thread_local! {
    static PERLIN_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_rng(&mut rand::rng()));
}

struct Perlin {
    randvec: [Coords; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    fn new() -> Self {
        let randvec = PERLIN_RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            std::array::from_fn(|_| Coords::random(&mut rng, -1.0..1.0))
        });
        Self {
            randvec,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    fn noise(&self, p: Coords) -> f32 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();

        let i = (p.x() - u) as i32;
        let j = (p.y() - v) as i32;
        let k = (p.z() - w) as i32;

        let mut c = [Coords::default(); 8];
        for (index, &(di, dj, dk)) in OFFSETS.iter().enumerate() {
            let ii = ((i + di) & 255) as usize;
            let jj = ((j + dj) & 255) as usize;
            let kk = ((k + dk) & 255) as usize;
            let perm_index = self.perm_x[ii] ^ self.perm_y[jj] ^ self.perm_z[kk];
            c[index] = self.randvec[perm_index];
        }
        Self::trilinear_interp(c, u, v, w)
    }

    fn turb(&self, mut p: Coords, depth: usize) -> f32 {
        let mut weight = 1.;
        (0..depth)
            .fold(0., |accum, _| {
                let result = accum + self.noise(p) * weight;
                p *= 2.;
                weight *= 0.5;
                result
            })
            .abs()
    }

    fn trilinear_interp(c: [Coords; 8], u: f32, v: f32, w: f32) -> f32 {
        let mut accum = 0.;
        let u = u * u * (3. - 2. * u);
        let v = v * v * (3. - 2. * v);
        let w = w * w * (3. - 2. * w);
        for (index, &(i, j, k)) in OFFSETS.iter().enumerate() {
            let weight_vec = Coords::new(u - i as f32, v - j as f32, w - k as f32);
            let weight_u = if i == 1 { u } else { 1. - u };
            let weight_v = if j == 1 { v } else { 1. - v };
            let weight_w = if k == 1 { w } else { 1. - w };
            accum += weight_u * weight_v * weight_w * c[index].dot(weight_vec);
        }
        accum
    }

    fn generate_perm() -> [usize; POINT_COUNT] {
        let mut p = std::array::from_fn(|i| i);
        PERLIN_RNG.with(|rng| {
            p.shuffle(&mut rng.borrow_mut());
        });
        p
    }
}
