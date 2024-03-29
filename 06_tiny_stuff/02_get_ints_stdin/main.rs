#![allow(dead_code)]

fn get_int<T>() -> T 
where T: std::str::FromStr,
    T::Err: std::fmt::Debug
{
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).expect("failed to read input.");
    buf.trim().parse::<T>().expect("invalid input")
}

fn read_vec<T>(n: usize) -> Vec<T>
where T: std::str::FromStr,
    T::Err: std::fmt::Debug {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).expect("failed to read input.");
    let mut res: Vec<T> = Vec::with_capacity(n);
    let divided = buf.trim().split_whitespace();
    for (i, s) in divided.enumerate() {
        let num: T = s.trim().parse::<T>().expect("invalid input");
        res.push(num);
        if i == n {
            break;
        }
    }
    res
}

fn get_2int<T>() -> (T, T)
where T: std::str::FromStr,
    T::Err: std::fmt::Debug
{
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).expect("failed to read input.");
    let mut devided = buf.trim().split_whitespace();
    let a = devided.next().unwrap().trim().parse::<T>().expect("invalid input");
    let b = devided.next().unwrap().trim().parse::<T>().expect("invalid input");
    (a, b)
}

fn get_3int<T>() -> (T, T, T)
where T: std::str::FromStr,
    T::Err: std::fmt::Debug
{
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).expect("failed to read input.");
    let mut devided = buf.trim().split_whitespace();
    let a = devided.next().unwrap().trim().parse::<T>().expect("invalid input");
    let b = devided.next().unwrap().trim().parse::<T>().expect("invalid input");
    let c = devided.next().unwrap().trim().parse::<T>().expect("invalid input");
    (a, b, c)
}

fn main() {
    let (n, mut k) = get_2int::<usize>();
    k += 1;
    let v = read_vec::<i64>(n);
    println!("{n}, {k}");
    println!("{v:?}");
}