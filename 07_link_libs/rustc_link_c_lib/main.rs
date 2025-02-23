
extern "C" {
    fn add(x: i32, y: i32) -> i32;
}

fn main() {
    let (x, y) = (23, 42);
    let z = unsafe { add(x, y) };
    println!("{}+{}={}", x, y, z);
}