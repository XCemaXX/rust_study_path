

fn prev(x: &i32) -> i32 {
    x - 1
}

fn next(x: &i32) -> i32 {
    x + 1
}

fn zipmap<Arg, Res, F: Fn(&Arg) -> Res>(funcs: &[F], args: &[Arg]) -> Vec<Res> {
    let iter = args.iter().zip(funcs);
    let mut res = Vec::with_capacity(iter.len());
    for (arg, func) in iter {
        res.push(func(arg));
    }
    res
}

fn main() {
    let funcs: Vec<fn(&i32) -> i32> = vec![next, prev, next];
    let args: Vec<i32> = vec![10, 20, 30];
    let res = zipmap(&funcs, &args);
    println!("{res:?}");
}