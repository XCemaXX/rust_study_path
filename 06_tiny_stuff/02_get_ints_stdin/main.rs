#![allow(dead_code)]

fn get_int<T: std::str::FromStr>() -> T {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    s.trim().parse().ok().unwrap()
}

fn read_vec<T: std::str::FromStr>(n: usize) -> Vec<T> {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    s.split_whitespace()
        .take(n)
        .map(|w| w.parse().ok().unwrap())
        .collect()
}

fn get_2int<T: std::str::FromStr>() -> (T, T) {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    let mut it = s.split_whitespace();
    (
        it.next().unwrap().parse().ok().unwrap(),
        it.next().unwrap().parse().ok().unwrap(),
    )
}

fn get_3int<T: std::str::FromStr>() -> (T, T, T) {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    let mut it = s.split_whitespace();
    (
        it.next().unwrap().parse().ok().unwrap(),
        it.next().unwrap().parse().ok().unwrap(),
        it.next().unwrap().parse().ok().unwrap(),
    )
}

fn main() {
    let (n, mut k) = get_2int::<usize>();
    k += 1;
    let v = read_vec::<i64>(n);
    println!("{n}, {k}");
    println!("{v:?}");
}
