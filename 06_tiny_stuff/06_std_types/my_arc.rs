use std::{
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{self, AtomicUsize, Ordering}, thread,
};

struct MyArc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}

unsafe impl<T: Sync + Send> Send for MyArc<T> {}
unsafe impl<T: Sync + Send> Sync for MyArc<T> {}

struct ArcInner<T> {
    rc: AtomicUsize,
    data: T,
}

impl<T> MyArc<T> {
    pub fn new(data: T) -> Self {
        let boxed = Box::new(ArcInner {
            rc: AtomicUsize::new(1),
            data,
        });
        MyArc {
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            phantom: PhantomData,
        }
    }

    fn inner(&self) -> &ArcInner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Deref for MyArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let inner = self.inner();
        &inner.data
    }
}

impl<T> Clone for MyArc<T> {
    fn clone(&self) -> Self {
        let inner = self.inner();
        let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);
        if old_rc >= isize::MAX as usize {
            std::process::abort();
        }

        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T> Drop for MyArc<T> {
    fn drop(&mut self) {
        let inner = self.inner();
        let old_rc = inner.rc.fetch_sub(1, Ordering::Release);
        if old_rc != 1 {
            return;
        }
        atomic::fence(Ordering::Acquire);
        unsafe { let _ = Box::from_raw(self.ptr.as_ptr()); }
    }
}

fn main() {
    let v = MyArc::new(vec![1, 2, 3, 4]);
    let mut handles = Vec::new();
    for _ in 1..5 {
        let v_ref = MyArc::clone(&v);
        handles.push( thread::spawn(move || {
            let thread_id = thread::current().id();
            println!("{:?} {:?}", thread_id, *v_ref);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    println!("{:?}", *v);
}
