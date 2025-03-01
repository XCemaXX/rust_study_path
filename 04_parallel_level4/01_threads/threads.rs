use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| { //async
        for i in 0..10 {
            println!("-Child {i}");
            thread::sleep(Duration::from_millis(5));
        }
    });

    let scope_s = String::from("Shared_string_by_ref");
    thread::scope(|s| { //sync to caller thread, async inside
        // can use ref, cause ref will live longer than join of threads
        s.spawn(|| {
            for i in 0..3 {
                println!("Scoped1: {} {}", scope_s, i);
                thread::sleep(Duration::from_millis(2));
            }
        });
        s.spawn(|| {
            for i in 0..3 {
                println!("Scoped2: {} {}", scope_s, i);
                thread::sleep(Duration::from_millis(10));
            }
        });
    });

    for i in 0..3 {
        println!("Parent {i}");
        thread::sleep(Duration::from_millis(5));
    }
    
    let h = handle.join();
    println!("{h:?}");

    let name = String::from("Named");
    let h = thread::Builder::new()
        .name(name.clone())
        .stack_size(32 * 1024)
        .spawn(move || {
            println!("{name}");
    })
    .unwrap()
    .join();
    println!("{h:?}");
}