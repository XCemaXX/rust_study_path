use std::panic;


fn get_pair_mut<T>(slice: &mut [T], index1: usize, index2: usize) -> (&mut T, &mut T) {
    let (mini, maxi) = (index1.min(index2), index1.max(index2));
    let (lo, hi) = slice.split_at_mut(maxi);
    (&mut lo[mini], &mut hi[0])
}

fn main() {
    let mut m = vec![1, 2, 3];
    let (a, b) = get_pair_mut(&mut m, 0, 2);
    *a = 20;
    *b = 30; 
    println!("{m:?}");
    let _ = panic::catch_unwind(|| {
        println!("Panic1");
        get_pair_mut(&mut m.clone(), 1, 1);
    });
    let _ = panic::catch_unwind(move || {
        println!("Panic2");
        get_pair_mut(&mut m, 3, 6);
    });
}