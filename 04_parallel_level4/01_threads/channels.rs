use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn simple_send_recv() {
    let (sender, receiver) = mpsc::channel();
    sender.send(10).unwrap();

    let sender2 = sender.clone();
    sender2.send(20).unwrap();

    sender.send(30).unwrap();

    println!("Receive {:?}", receiver.recv());
    println!("Receive {:?}", receiver.recv());
    println!("Receive {:?}", receiver.recv());
}

fn unbounded_async_channel() {
    let (sender, receiver) = mpsc::channel();

    let t = thread::spawn(move || {
        let thread_id = thread::current().id();
        for i in 0..10 {
            sender.send(format!("Async {i}")).unwrap();
            println!("Child async {i}");
        }
        println!("Child end {thread_id:?}");
    });
    thread::sleep(Duration::from_millis(100));
    
    for msg in receiver.iter() {
        println!("Parent {msg}");
    }
    t.join().unwrap();
}

fn bounded_sync_channel() {
    let (sender, receiver) = mpsc::sync_channel(2);
    let t = thread::spawn(move || {
        let thread_id = thread::current().id();
        for i in 0..10 {
            sender.send(format!("Sync {i}")).unwrap();
            println!("Child sync {i}");
        }
        println!("Child end {thread_id:?}");
    });
    thread::sleep(Duration::from_millis(100));
    for msg in receiver.iter() {
        println!("Parent {msg}");
    }
    t.join().unwrap();
}

fn main () {
    simple_send_recv();
    unbounded_async_channel();
    bounded_sync_channel();
}