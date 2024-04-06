use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| { //async
        for i in 0..10 {
            println!("Child {i}");
            thread::sleep(Duration::from_millis(5));
        }
    });

    let scope_s = String::from("Scope");
    thread::scope(|_scope| { //sync, cause of scope
        for i in 0..3 {
            println!("{} {}", scope_s, i);
            thread::sleep(Duration::from_millis(5));
        }
    });
    for i in 0..3 {
        println!("Parent {i}");
        thread::sleep(Duration::from_millis(5));
    }
    
    let h = handle.join();
    println!("{h:?}");
}