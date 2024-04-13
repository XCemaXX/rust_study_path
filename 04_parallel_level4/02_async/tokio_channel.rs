use tokio::sync::mpsc::{self, Receiver};

async fn ping_handler(mut input: Receiver<()>) {
    let mut count = 0_usize;

    while let Some(_) = input.recv().await {
        count += 1;
        println!("Get ping {count}");
    }
    println!("End");
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel(3);
    let task = tokio::spawn(ping_handler(receiver));
    for i in 1..=10 {
        sender.send(()).await.expect("fail to send ping");
        println!("Sended {i} pings");
    }
    println!("Will end");
    drop(sender);
    task.await.expect("Something went wrong");
}