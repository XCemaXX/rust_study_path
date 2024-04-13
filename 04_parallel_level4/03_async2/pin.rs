use tokio::sync::{mpsc, oneshot};
use tokio::task::spawn;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Task {
    data: u32,
    respond_on: oneshot::Sender<u32>,
}

async fn worker(mut work_queue: mpsc::Receiver<Task>) {
    let mut works_done = 0;
    let mut timeout_future = Box::pin(sleep(Duration::from_millis(50)));
    loop {
        tokio::select! {
            Some(work) = work_queue.recv() => {
                sleep(Duration::from_millis(10)).await;
                work.respond_on.send(work.data * 1000).expect("fail send resp");
                works_done += 1;
            },
            _ = &mut timeout_future => { 
                println!("Works done {works_done}"); 
                timeout_future = Box::pin(sleep(Duration::from_millis(50)));
            },
        }
    }
}

async fn do_work(work_queue: &mpsc::Sender<Task>, data: u32) -> u32 {
    let (sender, receiver) = oneshot::channel();
    work_queue.send(Task {data, respond_on: sender}).await.expect("fail to send work");
    receiver.await.expect("fail to wait response")
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel(5);
    spawn(worker(receiver));
    for i in 0..10 {
        let resp = do_work(&sender, i).await;
        println!("Result {i}: {resp}");
    }
}