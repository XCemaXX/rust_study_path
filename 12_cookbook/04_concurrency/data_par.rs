use rayon::prelude::*;

fn test_predicates() {
    let mut vec = vec![2, 4, 6, 8];

    assert!(!vec.par_iter().any(|n| (*n % 2) != 0));
    assert!(vec.par_iter().all(|n| *n < 9));

    vec.push(9);

    assert!(vec.par_iter().any(|n| (*n % 2) != 0));
    assert!(!vec.par_iter().all(|n| *n < 9));
}

fn search_elem() {
    let vec = vec![6, 1, 2, 9, 3, 8, 13];

    let f1 = vec.par_iter().find_any(|&&x| x == 9);
    assert_eq!(f1, Some(&9));

    let f3 = vec.par_iter().find_any(|&&x| x > 8);
    assert!(f3 > Some(&8));
    println!("Founded: {:?}", f3);
}

fn sort_vec() {
    let mut vec = vec![6, 1, 2, 9, 3, 8, 13];
    vec.par_sort_unstable();
    println!("Sorted {:?}", vec);
}

struct Person {
    age: u32,
}

fn complex_condition() {
    let persons = [23, 19, 42, 17, 17, 31, 30]
        .map(|age| Person { age })
        .into_iter()
        .collect::<Vec<_>>();

    let num_over_30 = persons.par_iter().filter(|&p| p.age > 30).count();
    let sum_over_30 = persons
        .par_iter()
        .map(|p| p.age)
        .filter(|&a| a > 30)
        .reduce(|| 0, |a, b| a + b);
    let alt_sum = persons
        .par_iter()
        .map(|p| p.age)
        .filter(|&a| a > 30)
        .sum::<u32>();
    assert_eq!(sum_over_30, alt_sum);
    let avg_over_30 = sum_over_30 as f32 / num_over_30 as f32;
    println!("The average age of people older than 30 is {}", avg_over_30);
}

fn main() {
    test_predicates();
    search_elem();
    sort_vec();
    complex_condition();
}
