use futures::executor::block_on;

async fn count_to(limit: usize) {
    for i in 1..=limit {
        print!("{i} ");
    }
    println!();
}

async fn async_main(limit: usize) {
    count_to(limit).await;
}

fn main () {
    let f = async_main(10);
    block_on(f);
}