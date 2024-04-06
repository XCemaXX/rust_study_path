use std::sync::{Arc, Mutex};
use std::thread;

fn arc_use() {
    let v = Arc::new(vec![1, 2, 3, 4]);
    let mut handles = Vec::new();
    for _ in 1..5 {
        let v_clone = Arc::clone(&v);
        handles.push( thread::spawn(move || {
            let thread_id = thread::current().id();
            println!("{:?} {:?}", thread_id, v_clone);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("{v:?}");
}

fn mutex_use() {
    let v = Arc::new(Mutex::new(vec![1000]));
    let mut handles = Vec::new();
    for i in 1..5 {
        let v_clone = Arc::clone(&v);
        handles.push( thread::spawn(move || {
            let mut v_in = v_clone.lock().unwrap();
            v_in.push(i);
            println!("{:?}", *v_in);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("parent {:?}", v.lock().unwrap());
}

fn main() {
    arc_use();
    mutex_use()
}