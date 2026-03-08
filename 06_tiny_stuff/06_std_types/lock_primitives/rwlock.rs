use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

use atomic_wait::{wait, wake_all, wake_one};

pub(super) const LOCKED_FOR_WRITE: u32 = u32::MAX;
const _: () = assert!(LOCKED_FOR_WRITE % 2 == 1);
pub(super) const MAX_READERS: u32 = u32::MAX - 2;

pub struct RwLock<T> {
    /// The number of read locks times two, plus one if there's a writer waiting.
    /// This means that readers may acquire the lock when
    /// the state is even, but need to block when odd.
    state: AtomicU32,
    writer_wake_counter: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
            writer_wake_counter: AtomicU32::new(0),
        }
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        let mut s = self.state.load(Ordering::Relaxed);
        loop {
            if s.is_multiple_of(2) {
                // Even, only readers
                assert!(s < MAX_READERS, "too many readers");
                match self.state.compare_exchange_weak(
                    s,
                    s + 2, // add reader x 2
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return ReadGuard { rwlock: self },
                    Err(e) => s = e,
                }
            }
            if s % 2 == 1 {
                // Odd, one writer waiting
                wait(&self.state, s);
                s = self.state.load(Ordering::Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        let mut s = self.state.load(Ordering::Relaxed);
        loop {
            // unlocked, no readers. Readers 2+
            if s <= 1 {
                match self.state.compare_exchange(
                    s,
                    LOCKED_FOR_WRITE,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return WriteGuard { rwlock: self },
                    Err(e) => {
                        s = e;
                        continue;
                    }
                }
            }
            // Lock new readers
            if s.is_multiple_of(2) {
                match self
                    .state
                    .compare_exchange(s, s + 1, Ordering::Relaxed, Ordering::Relaxed)
                {
                    Ok(_) => {}
                    Err(e) => {
                        s = e;
                        continue;
                    }
                }
            }
            let w = self.writer_wake_counter.load(Ordering::Acquire);
            s = self.state.load(Ordering::Relaxed);
            if s >= 2 {
                wait(&self.writer_wake_counter, w);
                s = self.state.load(Ordering::Relaxed);
            }
        }
    }
}

pub struct ReadGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.rwlock.state.fetch_sub(2, Ordering::Release) == 3 {
            // If we decremented from 3 to 1, that means
            // the RwLock is now unlocked _and_ there is
            // a waiting writer, which we wake up.
            self.rwlock
                .writer_wake_counter
                .fetch_add(1, Ordering::Release);
            wake_one(&self.rwlock.writer_wake_counter);
        }
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.rwlock.value.get() }
    }
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.state.store(0, Ordering::Release);
        self.rwlock
            .writer_wake_counter
            .fetch_add(1, Ordering::Release);
        wake_one(&self.rwlock.writer_wake_counter);
        wake_all(&self.rwlock.state);
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.rwlock.value.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.rwlock.value.get() }
    }
}
