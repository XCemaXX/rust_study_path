use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use atomic_wait::{wait, wake_all, wake_one};

use crate::lock_primitives::mutex::MutexGuard;

pub struct Condvar {
    counter: AtomicU32,
    num_waiters: AtomicUsize,
}

impl Condvar {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            num_waiters: AtomicUsize::new(0),
        }
    }

    pub fn notify_one(&self) {
        if self.num_waiters.load(Ordering::Relaxed) > 0 {
            self.counter.fetch_add(1, Ordering::Relaxed);
            wake_one(&self.counter);
        }
    }

    #[allow(dead_code)]
    pub fn notify_all(&self) {
        if self.num_waiters.load(Ordering::Relaxed) > 0 {
            self.counter.fetch_add(1, Ordering::Relaxed);
            wake_all(&self.counter);
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.num_waiters.fetch_add(1, Ordering::Relaxed);
        let counter = self.counter.load(Ordering::Relaxed);

        let mutex = guard.mutex;
        drop(guard);

        wait(&self.counter, counter);
        self.num_waiters.fetch_sub(1, Ordering::Relaxed);

        mutex.lock()
    }
}
