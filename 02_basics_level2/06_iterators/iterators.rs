

fn main() {
    let triplets = (1u32..)
        .flat_map(|z| (1..=z).map(move |y| (y, z)))
        .flat_map(|(y, z)| (1..=y).map(move |x| (x, y, z)))
        .filter(|(x, y, z) | x*x + y*y == z*z );
    let first_ten = triplets
        .take(10)
        .collect::<Vec<_>>();
    println!("{:?}", first_ten);

    let items = vec![2u16, 1, 0];
    let vec_options = items.iter()
        .map(|x| x.checked_sub(1))
        .collect::<Vec<_>>();
    let opt_vec  = items.iter()
        .map(|x| x.checked_sub(1))
        .collect::<Option<Vec<_>>>();
    println!("{vec_options:?} vs {opt_vec:?}");
    let items = vec![2u16, 1, 3];
    let vec_options = items.iter()
        .map(|x| x.checked_sub(1))
        .collect::<Vec<_>>();
    let opt_vec  = items.iter()
        .map(|x| x.checked_sub(1))
        .collect::<Option<Vec<_>>>();
    println!("{vec_options:?} vs {opt_vec:?}");
}