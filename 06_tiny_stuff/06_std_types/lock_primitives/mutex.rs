use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

use atomic_wait::{wait, wake_one};

mod state {
    pub(super) const UNLOCKED: u32 = 0;
    pub(super) const LOCKED_NO_WAITERS: u32 = 1;
    pub(super) const LOCKED_WITH_WAITERS: u32 = 2;
}

pub struct Mutex<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: state::UNLOCKED.into(),
            value: value.into(),
        }
    }

    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        if self
            .state
            .compare_exchange(
                state::UNLOCKED,
                state::LOCKED_NO_WAITERS,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            lock_contended(&self.state);
        }

        MutexGuard { mutex: self }
    }
}

#[cold]
fn lock_contended(state: &AtomicU32) {
    const MAX_SPINS: i32 = 100;
    let mut spin_count = 0;
    while state.load(Ordering::Relaxed) == state::LOCKED_NO_WAITERS && spin_count < MAX_SPINS {
        spin_count += 1;
        std::hint::spin_loop();
    }
    if state
        .compare_exchange(
            state::UNLOCKED,
            state::LOCKED_NO_WAITERS,
            Ordering::Acquire,
            Ordering::Relaxed,
        )
        .is_ok()
    {
        return;
    }
    while state.swap(state::LOCKED_WITH_WAITERS, Ordering::Acquire) != state::UNLOCKED {
        wait(state, state::LOCKED_WITH_WAITERS);
    }
}

pub struct MutexGuard<'a, T> {
    pub(crate) mutex: &'a Mutex<T>,
}

unsafe impl<'a, T> Sync for MutexGuard<'a, T> where T: Sync {}
unsafe impl<'a, T> Send for MutexGuard<'a, T> where T: Send {}

impl<T> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        if self.mutex.state.swap(state::UNLOCKED, Ordering::Release) == state::LOCKED_WITH_WAITERS {
            wake_one(&self.mutex.state);
        }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}
