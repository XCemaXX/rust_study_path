struct Fibonacci {
    cur: i64,
    next: i64
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { cur: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let new_next = self.next + self.cur;
        self.cur = self.next;
        self.next = new_next;
        Some(self.cur)
    }
}

fn offset_differences<N>(offset: usize, values: Vec<N>) -> Vec<N>
where
    N: Copy + std::ops::Sub<Output = N>,
{
    let len = values.len();
    (0..len)
        .map(|i| values[(i + offset) % len] - values[i])
        .collect::<Vec<_>>()
    //###or
    //let a = (&values).into_iter();
    //let b = (&values).into_iter().cycle().skip(offset);
    //a.zip(b).map(|(a, b)| *b - *a).take(values.len()).collect()
}


fn main() {
    let fib = Fibonacci::new();
    for (i, n) in fib.enumerate().take(5) {
        println!("fib({}): {}", i, n);
    }

    let res = offset_differences(1, vec![1, 3, 5, 7]);
    println!("{:?}", res);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_one() {
        assert_eq!(offset_differences(1, vec![1, 3, 5, 7]), vec![2, 2, 2, -6]);
        assert_eq!(offset_differences(1, vec![1, 3, 5]), vec![2, 2, -4]);
        assert_eq!(offset_differences(1, vec![1, 3]), vec![2, -2]);
    }

    #[test]
    fn test_larger_offsets() {
        assert_eq!(offset_differences(2, vec![1, 3, 5, 7]), vec![4, 4, -4, -4]);
        assert_eq!(offset_differences(3, vec![1, 3, 5, 7]), vec![6, -2, -2, -2]);
        assert_eq!(offset_differences(4, vec![1, 3, 5, 7]), vec![0, 0, 0, 0]);
        assert_eq!(offset_differences(5, vec![1, 3, 5, 7]), vec![2, 2, 2, -6]);
    }

    #[test]
    fn test_custom_type() {
        assert_eq!(
            offset_differences(1, vec![1.0, 11.0, 5.0, 0.0]),
            vec![10.0, -6.0, -5.0, 1.0]
        );
    }

    #[test]
    fn test_degenerate_cases() {
        assert_eq!(offset_differences(1, vec![0]), vec![0]);
        assert_eq!(offset_differences(1, vec![1]), vec![0]);
        let empty: Vec<i32> = vec![];
        assert_eq!(offset_differences(1, empty), vec![]);
    }
}