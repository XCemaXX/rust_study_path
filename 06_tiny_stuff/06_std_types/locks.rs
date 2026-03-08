mod lock_primitives;

use std::{
    thread,
    time::{Duration, Instant},
};

use lock_primitives::{Condvar, Mutex, RwLock};

fn bench_mutex() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    for _ in 0..5000000 {
        *m.lock() += 1;
    }
    let duration = start.elapsed();
    println!(
        "One thread mutex: locked {} times in {:?}",
        *m.lock(),
        duration
    );
}

fn bench_mutex_threads() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..20 {
            s.spawn(|| {
                for _ in 0..250000 {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration = start.elapsed();
    println!(
        "Multiple threads mutex: locked {} times in {:?}",
        *m.lock(),
        duration
    );
}

fn test_condvar() {
    let mutex = Mutex::new(0);
    let condvar = Condvar::new();

    let mut wakeups = 0;
    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_millis(250));
            *mutex.lock() = 123;
            condvar.notify_one();
        });
        let mut m = mutex.lock();
        while *m < 100 {
            m = condvar.wait(m);
            wakeups += 1;
        }
        assert_eq!(*m, 123);
    });

    // Check that the main thread actually did wait (not busy-loop),
    // while still allowing for a few spurious wake ups.
    assert!(wakeups < 10);
    println!("Condvar: wakeups {wakeups}");
}

fn bench_rwlock_threads() {
    let rw = RwLock::new(0);
    std::hint::black_box(&rw);

    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..20 {
            s.spawn(|| {
                for _ in 0..250_000 {
                    *rw.write() += 1;
                }
            });
        }
    });

    let duration = start.elapsed();
    println!(
        "Multiple threads rwlock write: locked {} times in {:?}",
        *rw.read(),
        duration
    );
}

fn bench_rwlock_read_heavy() {
    let rw = RwLock::new(0);
    std::hint::black_box(&rw);

    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..20 {
            s.spawn(|| {
                for j in 0..250_000 {
                    if j % 100 == 0 {
                        *rw.write() += 1;
                    } else {
                        std::hint::black_box(*rw.read());
                    }
                }
            });
        }
    });

    let duration = start.elapsed();
    println!(
        "RWLock read-heavy: final value {}, elapsed {:?}",
        *rw.read(),
        duration
    );
}

fn main() {
    bench_mutex();
    bench_mutex_threads();
    test_condvar();
    bench_rwlock_threads();
    bench_rwlock_read_heavy();
}
