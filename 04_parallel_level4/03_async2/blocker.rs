use futures::future::join_all;
use std::time::{Duration, Instant};

async fn sleep_seq_ms(start: &Instant, id: u64, duration_ms: u64) {
    std::thread::sleep(Duration::from_millis(duration_ms));
    println!("Future {} sleeps for {} ms, ended after {} ms", 
        id, duration_ms, start.elapsed().as_millis());
}

async fn sleep_ms(start: &Instant, id: u64, duration_ms: u64) {
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    println!("Future {} sleeps for {} ms, ended after {} ms", 
        id, duration_ms, start.elapsed().as_millis());
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let futures_failed = (1..=5).map(|t| sleep_seq_ms(&start, t, t * 10));
    join_all(futures_failed).await;
    println!("########");
    let start = Instant::now();
    let futures = (1..=5).map(|t| sleep_ms(&start, t, t * 10));
    join_all(futures).await;
}