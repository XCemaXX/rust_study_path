use tokio::time;

async fn count_to(limit: usize) {
    for i in 1..=limit {
        println!("{i} ");
        time::sleep(time::Duration::from_millis(5)).await;
    }
}

#[tokio::main]
async fn main() {
    let task = tokio::spawn(count_to(10));
    for i in 1..5 {
        println!("Counter in main: {i}");
        time::sleep(time::Duration::from_millis(5)).await;
    }
    println!("Waiting");
    let _ = tokio::join!(task);
}