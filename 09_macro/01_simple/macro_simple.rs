
macro_rules! squared_broken {
    ($e:expr ) => {
        $e * $e
    };
}

macro_rules! squared {
    ($e:expr) => {{
        let val = $e;
        val * val
    }};
}

macro_rules! gen_vector {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(temp_vec.push($x);)*
            temp_vec
        }
    };
    ( $( $x:expr )=>* ) => {
        {
            let mut temp_vec = Vec::new();
            $(temp_vec.push($x);)*
            $(temp_vec.push($x);)*
            temp_vec
        }
    };
}

macro_rules! not_change_global_scope {
    () => {
        let x = 42;
        print!("macro1: {x} ");
    };
}

macro_rules! declare_var {
    ($var:ident) => {
        let $var = 100;
        print!("macro2: {} ", $var);
    };
}

fn main() {
    println!("{:?}", squared!(1 + 1)); // [1+1]*[1+1]. not like in C: 1+1*1+1

    let sq = squared_broken!(({print!("call twice, "); 3}));
    println!("{sq}");
    let sq = squared!({print!("call once, "); 4});
    println!("{sq}");
    
    let c = gen_vector!(1+1, 23, 42);
    let d = gen_vector!["1+1".to_string(), "23".to_string()];
    let double_c = gen_vector!( 1 + 1 => 23 => 42);
    println!("{c:?} {d:?} {double_c:?}");

    let x = 23;
    not_change_global_scope!();
    println!("main1: {x}");
    declare_var!(x);
    println!("main2: {x}");
}