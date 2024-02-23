fn fib(n: u32) -> u32 {
    if n <= 2 { 
        1
    } else {
        fib(n-1) + fib(n-2)
    }
}

fn check_switch(n: char) {
    match n {
        'q'                         => println!("quit"),
        'w' | 'a' | 's' | 'd'       => println!("move"),
        '0' ..='9'                  => println!("num"),
        key if key.is_lowercase()   => println!("letter lower case: {key}"),
        _                           => println!("other"),
    }
}

fn main() {
    let n = 20;
    println!("fib({}) = {}", n, fib(n));
    assert_eq!(fib(n), 6765);

    check_switch('q');
    check_switch('a');
    check_switch('4');
    check_switch('j');
    check_switch('B');
}