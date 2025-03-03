
use squared_macro::squared;

fn main() {
    println!("{:?}", squared!(1 + 1)); // [1+1]*[1+1]. not like in C: 1+1*1+1

    let sq = squared!({print!("call once, "); 4});
    println!("{sq}");
}