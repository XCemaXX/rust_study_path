
fn run_with_log(call_back: impl FnOnce(i32) -> i32, arg: i32) -> i32 {
    println!("func call with arg: {arg}");
    call_back(arg)
}

fn make_greeter(prefix: String) -> impl Fn(&str) {
    // move allows catch by value
    return move |name| println!("{} {}", prefix, name);
}

fn main() {
    let fn3 = |x: i32| x + 3;
    println!("fn3 result {}", run_with_log(fn3, 10));
    println!("fn3 result {}", run_with_log(fn3, 20));

    let mut vec = Vec::new();
    // catch in lamda by reference
    let mut accumulate = |x| {
        vec.push(x);
        vec.iter().sum::<i32>()
    };
    println!("accumulate result {}", run_with_log(&mut accumulate, 2));
    println!("accumulate result {}", run_with_log(&mut accumulate, 3));

    let multiply = |x| x * vec.iter().sum::<i32>();
    println!("accumulate result {}", run_with_log(multiply, 6));

    let hi = make_greeter("Hello".to_string());
    hi("world");
}