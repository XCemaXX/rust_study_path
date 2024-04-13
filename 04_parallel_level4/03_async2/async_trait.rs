use async_trait::async_trait;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use rand::Rng;

#[async_trait]
trait Sleeper {
    async fn sleep(&self);
}

struct FixedSleeper {
    sleep_ms: u64,
}

struct RandomSleeper {}

#[async_trait]
impl Sleeper for FixedSleeper {
    async fn sleep(&self) {
        sleep(Duration::from_millis(self.sleep_ms)).await;
    }
}

#[async_trait]
impl Sleeper for RandomSleeper {
    async fn sleep(&self) {
        let num = rand::thread_rng().gen_range(0..100);
        sleep(Duration::from_millis(num)).await;
    }
}

async fn run_sleepers_multiple_times(
    sleepers: Vec<Box<dyn Sleeper>>,
    n_times: usize,
) {
    for i in 1..=n_times {
        println!("Running all sleepers. Iter {i}");
        for s in &sleepers {
            let start = Instant::now();
            s.sleep().await;
            println!("slept for {}ms", start.elapsed().as_millis());
        }
    }
}

#[tokio::main]
async fn main() {
    let sleepers: Vec<Box<dyn Sleeper>> = vec![
        Box::new(FixedSleeper {sleep_ms: 50}),
        Box::new(FixedSleeper {sleep_ms: 100}),
        Box::new(RandomSleeper {}),
        Box::new(RandomSleeper {}),
    ];
    run_sleepers_multiple_times(sleepers, 5).await;
}