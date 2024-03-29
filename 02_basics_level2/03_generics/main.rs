#![allow(dead_code)]
struct Point<T> {
    x: T, 
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Self{x: x, y: y}
    }
    fn coords(&self) -> (&T, &T) {
        (&self.x, &self.y)
    }
}

// #############
fn pick<T>(b: bool, even: T, odd: T) -> T {
    if b { even } else { odd }
}

// #############
#[derive(Debug, Clone)]
struct PointClonable<T> {
    x: T, 
    y: T,
}

//fn duplicate<T: Clone + Debug>(a: T) -> (T, T) { // the same
fn duplicate<T>(a: T) -> (T, T)
    where T: Clone, 
{
    (a.clone(), a.clone())
}

// #############
// fn add_42_millions<T: Into<i32>>(x: T) -> i32 { // the same
fn add_42_millions(x: impl Into<i32>) -> i32 { // trait Into<i32> ~ cast in C++
    x.into() + 42_000_000
}

fn pair_of(x: u32) -> impl std::fmt::Debug {
    (x + 1, x - 1)
}
fn main() {
    println!("first: {:?}", pick(true, 222, 333));
    println!("second: {:?}", pick(false, ("dog", 1), ("cat", 2)));

    let pi = Point::new(1, 2);
    let pf = Point::new(1.0, 2.0);
    println!("{:?} {:?}", pi.coords(), pf.coords());
    // duplicate(pi); //   Clone not implemented
    let clone = duplicate(PointClonable{x:1, y:2});
    println!("{:?}", clone); 

    let many = add_42_millions(123_i8); //_i8 type of int
    println!("{many}");
    println!("debuggable: {:?}", pair_of(27));
}