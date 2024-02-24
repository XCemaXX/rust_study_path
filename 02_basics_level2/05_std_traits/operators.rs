#[derive(Debug, Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

impl std::ops::Add<(i32, i32)> for Point {
    type Output = Self;
    fn add(self, other: (i32, i32)) -> Self {
        Self { x: self.x + other.0, y: self.y + other.1 }
    }
}

fn main() {
    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 100, y: 200 };
    let p3 = (1000, 3000);
    println!("{:?} + {:?} = {:?}", p1, p2, p1 + p2);
    println!("{:?} + {:?} = {:?}", p1, p3, p1 + p3);
}