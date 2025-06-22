#![allow(dead_code)]
use rand::{
    Rng,
    distr::{Distribution, StandardUniform, Uniform, Alphanumeric},
};
use rand_distr::Normal;

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Distribution<Point> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point {
        let (x, y) = rng.random();
        Point { x, y }
    }
}

fn main() {
    let mut rng = rand::rng();
    println!("Random: {} {}", rng.random::<f64>(), rng.random::<i32>());
    println!("Range: {}", rng.random_range(0.0..1000.));

    let die = Uniform::try_from(1..7).unwrap();
    loop {
        let throw = die.sample(&mut rng);
        println!("Roll the die: {}", throw);
        if throw == 6 {
            break;
        }
    }

    let normal = Normal::new(2.0, 3.0).unwrap();
    println!("{} from a N(2, 9) distribution", normal.sample(&mut rng));

    println!("Random tuple: {:?}", rng.random::<(i32, bool, f64)>());
    println!("Random point: {:?}", rng.random::<Point>());

    let pass: String = rng.clone().sample_iter(&Alphanumeric).take(30).map(char::from).collect();
    println!("Generated pass: {}", pass);

    const CHARSET: &[u8] = b"QWErty123890";
    let pass: String = (0..30).map(|_| {
        let idx = rng.random_range(0..CHARSET.len());
        CHARSET[idx] as char
    }).collect();
    println!("Generated from CHARSET pass: {}", pass);
}
