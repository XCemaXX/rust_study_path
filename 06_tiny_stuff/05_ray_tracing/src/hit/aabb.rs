use crate::{coords::Coords, ray::Ray, vec3::Axis};
use std::ops::{Add, Index, Range};

#[derive(Clone)]
pub struct Aabb {
    x: Range<f32>,
    y: Range<f32>,
    z: Range<f32>,
}

impl Aabb {
    pub fn new(x: Range<f32>, y: Range<f32>, z: Range<f32>) -> Self {
        let delta = 0.0001;
        Self {
            x: expand(x, delta),
            y: expand(y, delta),
            z: expand(z, delta),
        }
    }

    pub fn from_points(a: Coords, b: Coords) -> Self {
        let x = a.x().min(b.x())..a.x().max(b.x());
        let y = a.y().min(b.y())..a.y().max(b.y());
        let z = a.z().min(b.z())..a.z().max(b.z());
        Self::new(x, y, z)
    }

    pub fn from_boxes(a: Self, b: Self) -> Self {
        Self {
            x: a.x.start.min(b.x.start)..a.x.end.max(b.x.end),
            y: a.y.start.min(b.y.start)..a.y.end.max(b.y.end),
            z: a.z.start.min(b.z.start)..a.z.end.max(b.z.end),
        }
    }

    pub fn hit(&self, r: &Ray, ray_t: Range<f32>) -> Option<Range<f32>> {
        let ray_orig = r.origin();
        let ray_dir = r.direction();

        let mut start = ray_t.start;
        let mut end = ray_t.end;
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            let ax = &self[axis];
            let adinv = 1.0 / ray_dir[axis];

            let t0 = (ax.start - ray_orig[axis]) * adinv;
            let t1 = (ax.end - ray_orig[axis]) * adinv;

            start = start.max(t0.min(t1));
            end = end.min(t0.max(t1));

            if end <= start {
                return None;
            }
        }
        Some(start..end)
    }

    pub fn empty() -> Self {
        let empty = f32::MAX..f32::MIN;
        Self {
            x: empty.clone(),
            y: empty.clone(),
            z: empty.clone(),
        }
    }

    pub fn longest_axis(&self) -> Axis {
        use Axis::*;
        [
            (X, self.x.end - self.x.start),
            (Y, self.y.end - self.y.start),
            (Z, self.z.end - self.z.start),
        ]
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
    }

    pub fn corners(&self) -> [Coords; 8] {
        let [x_min, x_max] = [self.x.start, self.x.end];
        let [y_min, y_max] = [self.y.start, self.y.end];
        let [z_min, z_max] = [self.z.start, self.z.end];

        [x_min, x_max]
            .into_iter()
            .flat_map(|x| [y_min, y_max].into_iter().map(move |y| (x, y)))
            .flat_map(|(x, y)| {
                [z_min, z_max]
                    .into_iter()
                    .map(move |z| Coords::new(x, y, z))
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

impl Index<Axis> for Aabb {
    type Output = Range<f32>;

    fn index(&self, index: Axis) -> &Self::Output {
        use Axis::*;
        match index {
            X => &self.x,
            Y => &self.y,
            Z => &self.z,
        }
    }
}

fn expand(r: Range<f32>, delta: f32) -> Range<f32> {
    let padding = delta / 2.0;
    (r.start - padding)..(r.end + padding)
}

impl Add<Coords> for Aabb {
    type Output = Aabb;
    fn add(self, offset: Coords) -> Self::Output {
        Aabb::new(
            displace(self.x, offset.x()),
            displace(self.y, offset.y()),
            displace(self.z, offset.z()),
        )
    }
}

fn displace(r: Range<f32>, displacement: f32) -> Range<f32> {
    (r.start + displacement)..(r.end + displacement)
}
