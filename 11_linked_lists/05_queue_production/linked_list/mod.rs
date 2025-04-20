mod cursor;
mod iters;
mod lib_traits;
mod send_sync;

use std::{marker::PhantomData, ptr::NonNull};

pub use iters::{IntoIter, Iter, IterMut};
pub use cursor::CursorMut;

struct Node<T> {
    elem: T,
    front: Link<T>,
    back: Link<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    ////////////////////////// Front

    pub fn push_front(&mut self, elem: T) {
        let mut node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                elem,
                front: None,
                back: None,
            })))
        };
        match &mut self.front {
            None => self.back = Some(node),
            Some(old) => unsafe {
                old.as_mut().front = Some(node);
                node.as_mut().back = Some(*old);
            },
        }
        self.front = Some(node);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let node_ptr = self.front?;
        let boxed_node = unsafe { Box::from_raw(node_ptr.as_ptr()) };
        let result = boxed_node.elem;

        self.front = boxed_node.back;
        match &mut self.front {
            Some(new_head) => unsafe {
                new_head.as_mut().front = None;
            },
            None => {
                self.back = None;
            }
        }
        self.len -= 1;
        Some(result)
    }

    pub fn front(&self) -> Option<&T> {
        let node = self.front?;
        unsafe { Some(&node.as_ref().elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        let mut node = self.front?;
        unsafe { Some(&mut node.as_mut().elem) }
    }

    ////////////////////////// Back
    pub fn push_back(&mut self, elem: T) {
        let mut node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                elem,
                front: None,
                back: None,
            })))
        };
        match &mut self.back {
            None => self.front = Some(node),
            Some(old) => unsafe {
                old.as_mut().back = Some(node);
                node.as_mut().front = Some(*old);
            },
        }
        self.back = Some(node);
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let node_ptr = self.back?;
        let boxed_node = unsafe { Box::from_raw(node_ptr.as_ptr()) };
        let result = boxed_node.elem;

        self.back = boxed_node.front;
        match &mut self.back {
            Some(new_head) => unsafe {
                new_head.as_mut().back = None;
            },
            None => {
                self.front = None;
            }
        }
        self.len -= 1;
        Some(result)
    }

    pub fn back(&self) -> Option<&T> {
        let node = self.back?;
        unsafe { Some(&node.as_ref().elem) }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        let mut node = self.back?;
        unsafe { Some(&mut node.as_mut().elem) }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}