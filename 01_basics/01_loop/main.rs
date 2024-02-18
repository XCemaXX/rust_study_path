#![allow(unused)]

fn collatz_length(mut n: i32) -> u32 {
    let mut counter: u32 = 1;
    while n != 1 {
        n = if n % 2 == 1 { n * 3 + 1 } else { n / 2 };
        counter += 1;
    }
    counter
}

fn collatz_length_loop(mut n: i32) -> u32 {
    let mut counter: u32 = 1;
    loop {
        if n == 1 {
            break counter;
        } else if n % 2 == 1 {
            n = n * 3 + 1;
        } else {
            n = n / 2;
        }
        counter += 1;
    }
}

fn main() {
    println!("collatz_length: {}", collatz_length(11));
    assert_eq!(collatz_length(11), 15);
}