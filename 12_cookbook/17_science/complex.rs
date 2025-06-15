use std::f64::consts::PI;

use num::Complex;

fn main() {
    let c1 = Complex::new(1., 2.);
    let c2 = Complex::new(30., 40.);
    println!("Sum: {}", c1 + c2);

    let x = Complex::new(0.0, 2.0 * PI);
    println!("e^(2i * pi) = {}", x.exp());
}
