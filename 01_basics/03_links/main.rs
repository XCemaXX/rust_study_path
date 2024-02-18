fn magnitude(vec: &[f64]) -> f64 {
    let mut res = 0.0;
    for e in vec {
        res += e*e
    }
    res.sqrt()
}

fn normalize(vec : &mut [f64]) {
    let m = magnitude(&vec);
    for e in vec.iter_mut() {
        *e = *e / m;
    }
}

fn main() {
    println!("magnitude : {}", magnitude(&[0.0, 1.0, 0.0]));

    let mut v = [1.0, 2.0, 9.0];
    println!("magnitude {v:?}: {}", magnitude(&v));
    normalize(&mut v);
    println!("magnitude {v:?} after normalization: {}", magnitude(&v));
}