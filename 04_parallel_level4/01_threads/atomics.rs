use::std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    let mut counter = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..5 {
            s.spawn(|| {
                let prev_val = counter.fetch_add(1, Ordering::SeqCst);
                println!("{}", prev_val);
            });
        }
        let prev_val = counter.fetch_add(1, Ordering::SeqCst);
        println!("Main: {}", prev_val);
    });
    println!("Counter {}", *counter.get_mut());
}