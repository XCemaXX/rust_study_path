use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

fn arc_use() {
    println!("#############ARC");
    let v = Arc::new(vec![1, 2, 3, 4]);
    let mut handles = Vec::new();
    for _ in 1..5 {
        let v_ref = Arc::clone(&v);
        handles.push( thread::spawn(move || {
            let thread_id = thread::current().id();
            println!("{:?} {:?}", thread_id, v_ref);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("{v:?}");
}

fn arc_mut() {
    println!("#############ARC_MUT");
    let mut v = Arc::new(vec![10, 20, 30, 40]);
    let mut handles = Vec::new();
    for i in 1..5 {
        let mut v_ref = Arc::clone(&v);
        handles.push( thread::spawn(move || {
            let thread_id = i;
            let v = Arc::make_mut(&mut v_ref);
            v.push(i);
            println!("Clone-on-write v: {:?} {:?}", thread_id, v);
            thread::sleep(Duration::from_millis(500));
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("Origin v: {v:?}");
}

fn mutex_use() {
    println!("#############MUTEX");
    let v = Arc::new(Mutex::new(vec![1000]));
    let mut handles = Vec::new();
    for i in 1..5 {
        let v_ref = Arc::clone(&v);
        handles.push( thread::spawn(move || {
            let mut v_in = v_ref.lock().unwrap();
            v_in.push(i);
            println!("{:?}", *v_in);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("parent {:?}", v.lock().unwrap());
}

fn mut_vec(v: &RwLock<Vec<i32>>, val: i32) { // like RefCell threadsafe
    {
        let guard = v.write();
        //let mut b2 = v.write(); // will panic in runtime not compile time
        //let read_borrow = v.read(); // will panic in runtime not compile time
        guard.unwrap().push(val);
    }
    let read_borrow = v.read().unwrap();
    let read_borrow2 = v.read().unwrap();
    println!("1read: {read_borrow:?}");
    println!("2read: {read_borrow2:?}");
}

fn rwlock_use() {
    println!("#############RWLOCK");
    let ref_v = RwLock::new(vec![1, 2, 3, 4, 5]);

    std::thread::scope(|s| {
        for i in 0..3 {
            let ref_v = &ref_v;
            s.spawn(move || {
                mut_vec(ref_v, i);
            });
        }
    });

    println!("Changed via RwLock: {:?}", ref_v.read());
}

fn main() {
    arc_use();
    arc_mut();
    mutex_use();
    rwlock_use();
}