use num::{One, bigint::BigInt};

fn factorial(x: i32) -> BigInt {
    let mut result = BigInt::one();
    for i in 2..=x {
        result *= i;
    }
    result
}

fn main() {
    println!("{}! = {}", 1000, factorial(1000));
}
