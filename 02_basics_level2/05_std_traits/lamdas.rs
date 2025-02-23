use std::fmt::Display;


fn run_with_log(call_back: impl FnOnce(i32) -> i32, arg: i32) -> i32 {
    println!("1log: func call with arg: {arg}");
    call_back(arg)
}

fn run_with_log2<Args: Display, Res, Func: FnOnce(Args) -> Res>(call_back: Func, args: Args) -> Res {
    println!("2log: func call with arg: {args}");
    call_back(args)
}

fn run_two_times<Args, Res>(call_back: impl Fn(&Args) -> Res, args: &Args) -> (Res, Res) {
    let r1 = call_back(args);
    let r2 = call_back(args);
    (r1, r2)
}

fn run_two_times_mut(mut call_back: impl FnMut(i32)->()) {
    call_back(2);
    call_back(8);
}

fn make_greeter(prefix: String) -> impl Fn(&str) {
    // move allows catch by value
    return move |name| println!("{} {}", prefix, name);
}

fn main() {
    let fn3 = |x: i32| x + 3;
    println!("fn3 result {}", run_with_log(fn3, 10));
    println!("fn3 result {}", run_with_log2(fn3, 20));

    let mut vec = Vec::new();
    // catch in lamda by reference
    let mut accumulate = |x| {
        vec.push(x);
        vec.iter().sum::<i32>()
    };
    println!("accumulate result {}", run_with_log(&mut accumulate, 2));
    println!("accumulate result {}", run_with_log2(&mut accumulate, 3));

    let multiply = |x| x * vec.iter().sum::<i32>();
    println!("accumulate result {}; {:?}", run_with_log(multiply, 6), vec);

    //
    let fn_call2 = |(x, y): &(i32, i32)| { let r = x + y; println!("{r}"); r};
    run_two_times(fn_call2, &(3, 5));

    //
    let mut count = 0;
    let increment = |x| { count += x; println!("Add to counter {x}; Counter: {count}"); };
    run_two_times_mut(increment);
    println!("Counter: {count}");

    //
    let hi = make_greeter("Hello".to_string());
    hi("world");

    // move in lambda
    let has_gc = {
        let no_gc = vec!["C", "C++", "Rust"];
        move |lang: &str| -> bool {
            !no_gc.contains(&lang)
        }
    };
    let lang = "Java";
    println!("{} has garbage collector? {}", lang, has_gc(lang));
}