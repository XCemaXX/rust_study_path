use std::{thread, time::Duration};

fn find_max(arr: &[i32]) -> Option<i32> {
    if arr.len() < 3 {
        return arr.iter().max().copied();
    }

    let mid = arr.len() / 2;
    let (l, r) = arr.split_at(mid);

    crossbeam::scope(|s| {
        let thread1 = s.spawn(|_| find_max(l));
        let thread2 = s.spawn(|_| find_max(r));

        let max1 = thread1.join().unwrap()?;
        let max2 = thread2.join().unwrap()?;

        Some(max1.max(max2))
    })
    .unwrap()
}

fn producer_consumer() {
    let (producer_send, worker_recv) = crossbeam_channel::bounded(1);
    let (worker_send, consumer_recv) = crossbeam_channel::bounded(1);

    crossbeam::scope(|s| {
        s.spawn(|_| {
            for i in 0..6 {
                producer_send.send(i).unwrap();
                println!("Sent {}", i);
            }
            drop(producer_send);
        });

        for _ in 0..2 {
            let (sender, reciver) = (worker_send.clone(), worker_recv.clone());
            s.spawn(move |_| {
                thread::sleep(Duration::from_millis(500));
                for msg in reciver.iter() {
                    println!("Worker {:?} got: {}", thread::current().id(), msg);
                    sender.send(msg * 2).unwrap();
                }
            });
        }

        drop(worker_send);

        for msg in consumer_recv.iter() {
            println!("Consumer recv: {}", msg);
        }
    })
    .unwrap();
}

fn main() {
    let arr = [1, 25, -4, 10];
    let max = find_max(&arr);
    assert_eq!(max, Some(25));

    producer_consumer();
}
