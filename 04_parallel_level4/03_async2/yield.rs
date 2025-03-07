use futures::future::join_all;
use std::{future::Future, pin::Pin};

async fn pong() {
    for i in 0..5 {
        print!("pong{i} ");
        tokio::task::yield_now().await;
    }
}

async fn ping() {
    for i in 0..5 {
        print!("ping{i} ");
        tokio::task::yield_now().await;
        print!("PING{i} ");
        tokio::task::yield_now().await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let ping = ping();
    let pong = pong();
    let futures: Vec<Pin<Box<dyn Future<Output = ()>>>>  = vec![Box::pin(ping), Box::pin(pong)];
    join_all(futures).await;
    println!();
}