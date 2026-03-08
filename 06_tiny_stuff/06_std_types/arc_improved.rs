// from Rust Atomics and Locks

use std::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{fence, AtomicUsize, Ordering},
};

const WEAK_LOCKED: usize = usize::MAX;

struct ArcData<T> {
    // strong
    data_ref_count: AtomicUsize,
    // weak + 1strong if any
    alloc_ref_count: AtomicUsize,
    data: UnsafeCell<ManuallyDrop<T>>,
}

pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Weak<T> {}
unsafe impl<T: Send + Sync> Sync for Weak<T> {}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        // Safety: we used box leak
        unsafe { self.ptr.as_ref() }
    }

    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut n = self.data().alloc_ref_count.load(Ordering::Relaxed);
        loop {
            if n == 0 {
                return None;
            }
            assert!(n <= usize::MAX / 2);
            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(
                n,
                n + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                n = e;
                continue;
            }
            return Some(Arc { ptr: self.ptr });
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX / 2 {
            std::process::abort()
        }
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Ordering::Release) == 1 {
            fence(Ordering::Acquire);
            // Safety: we used box leak ic ctor
            unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
        }
    }
}

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Self {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                data: UnsafeCell::new(ManuallyDrop::new(data)),
                data_ref_count: 1.into(),
                alloc_ref_count: 1.into(),
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        // Safety: we used box leak
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc
            .data()
            .alloc_ref_count
            .compare_exchange(1, WEAK_LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }
        let is_unique = arc.data().data_ref_count.load(Ordering::Relaxed) == 1;
        arc.data().alloc_ref_count.store(1, Ordering::Release);
        if !is_unique {
            return None;
        }
        fence(Ordering::Acquire);
        let data = arc.data().data.get();
        // Safety: Nothing else can access the data, since
        // there's only one Arc, to which we have exclusive access.
        unsafe { Some(&mut *data) }
    }

    pub fn downgrade(&self) -> Weak<T> {
        let mut n = self.data().alloc_ref_count.load(Ordering::Relaxed);
        loop {
            if n == WEAK_LOCKED {
                std::hint::spin_loop();
                n = self.data().alloc_ref_count.load(Ordering::Relaxed);
            }
            assert!(n <= usize::MAX / 2);
            if let Err(e) = self.data().alloc_ref_count.compare_exchange_weak(
                n,
                n + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                n = e;
                continue;
            }
            return Weak { ptr: self.ptr };
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.data().data.get();
        // Safety: there is strong ptr
        unsafe { &*ptr }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        if self.data().data_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX / 2 {
            std::process::abort()
        }
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Ordering::Release) == 1 {
            fence(Ordering::Acquire);
            let ptr = self.data().data.get();
            // Safety: there is no more strong refs
            unsafe {
                ManuallyDrop::drop(&mut *ptr);
            }
            // subs 1 in alloc_ref_count that represented all strong refs
            drop(Weak { ptr: self.ptr })
        }
    }
}

static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

struct DetectDrop;
impl Drop for DetectDrop {
    fn drop(&mut self) {
        NUM_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    const DATA: &str = "data";
    let x = Arc::new((DATA, DetectDrop));
    let y = x.downgrade();
    let z = x.downgrade();

    let t = std::thread::spawn(move || {
        let y = y.upgrade().unwrap();
        assert_eq!(y.0, DATA)
    });
    assert_eq!(x.0, DATA);
    t.join().unwrap();

    assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);
    assert!(z.upgrade().is_some());
    drop(x);
    assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
    assert!(z.upgrade().is_none());
}
