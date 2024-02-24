use std::collections::HashMap;
use std::hash::Hash;
struct Counter<T: Eq + Hash> {
    values: HashMap<T, u64>,
}

impl<T: Eq + Hash> Counter<T> {
    fn new() -> Self {
        Counter{values: HashMap::new() }
    }

    fn count(&mut self, value: T) {
        *self.values.entry(value).or_insert(0) += 1;
    }

    fn times_seen(&self, value: T) -> u64 {
        self.values.get(&value).unwrap_or(&0).clone()
    }
}

fn main() {
    let mut ctr = Counter::new();
    ctr.count(13);
    ctr.count(14);
    ctr.count(16);
    ctr.count(14);
    ctr.count(14);
    ctr.count(11);

    for i in 10..20 {
        println!("saw {} values equal to {}", ctr.times_seen(i), i);
    }

    let mut strctr = Counter::new();
    strctr.count("apple");
    strctr.count("orange");
    strctr.count("apple");
    println!("got {} apples", strctr.times_seen("apple"));
}