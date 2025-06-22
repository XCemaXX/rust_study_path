use std::{cmp::Ordering, collections::HashMap};

fn mean(data: &[i32]) -> Option<f32> {
    let count = data.len();
    if count == 0 {
        return None;
    }
    let sum = data.iter().sum::<i32>() as f32;
    Some(sum / count as f32)
}

fn partition(data: &[i32]) -> Option<(Vec<i32>, i32, Vec<i32>)> {
    if data.len() == 0 {
        return None;
    }
    let (pivot_slice, tail) = data.split_at(1);
    let pivot = pivot_slice[0];
    let (left, right) = tail
        .iter()
        .fold((vec![], vec![]), |(mut left, mut right), &next| {
            {
                if next < pivot {
                    left.push(next);
                } else {
                    right.push(next);
                }
            }
            (left, right)
        });
    Some((left, pivot, right))
}

fn select(data: &[i32], k: usize) -> Option<i32> {
    if let Some((left, pivot, right)) = partition(data) {
        let pivot_idx = left.len();
        match pivot_idx.cmp(&k) {
            Ordering::Less => Some(pivot),
            Ordering::Equal => select(&left, k),
            Ordering::Greater => select(&right, k - (pivot_idx + 1)),
        }
    } else {
        None
    }
}

fn median(data: &[i32]) -> Option<f32> {
    let size = data.len();
    match size {
        odd if odd % 2 == 1 => select(data, odd / 2).map(|x| x as f32),
        even => {
            let med1 = select(data, (even / 2) - 1);
            let med2 = select(data, even / 2);

            match (med1, med2) {
                (Some(med1), Some(med2)) => Some((med1 + med2) as f32 / 2.0),
                _ => None,
            }
        }
    }
}

fn mode(data: &[i32]) -> Option<i32> {
    let freqs = data.iter().fold(HashMap::new(), |mut freqs, v| {
        *freqs.entry(v).or_insert(0) += 1;
        freqs
    });
    freqs
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(v, _)| *v)
}

fn std_deviation(data: &[i32]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|v| {
                    let diff = mean - (*v as f32);
                    diff * diff
                })
                .sum::<f32>()
                / count as f32;
            Some(variance.sqrt())
        }
        _ => None,
    }
}

fn zscore(data: &[i32], k: usize) -> Option<f32> {
    match (mean(data), std_deviation(data)) {
        (Some(mean), Some(std_deviation)) => {
            let diff = data[k] as f32 - mean;
            Some(diff / std_deviation)
        }
        _ => None,
    }
}

fn main() {
    let data = [3, 1, 6, 1, 5, 8, 1, 8, 10, 11];
    println!("Data {:?}", data);
    println!("Mean of the data is {:?}", mean(&data));
    println!("Partition is {:?}", partition(&data));
    println!("Selection at ordered index {} is {:?}", 5, select(&data, 5));
    println!("Median is {:?}", median(&data));
    println!("Mode of the data is {:?}", mode(&data));
    println!("Standard deviation is {:?}", std_deviation(&data));
    const I: usize = 4;
    println!(
        "Z-score of data at index {} (with value {}) is {:?}",
        I,
        data[I],
        zscore(&data, I)
    );
}
