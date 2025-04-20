// mutable singly-linked list on unsafe

// check with "cargo +nightly miri test --bin linked_lists_fail_unsafe"
use std::ptr;

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

impl<T> List<T> {
    fn new() -> Self {
        Self {
            head: None,
            tail: ptr::null_mut(),
        }
    }

    fn push(&mut self, elem: T) {
        let mut node = Box::new(Node { elem, next: None });
        let node_ptr: *mut _ = &mut *node;

        if self.tail.is_null() {
            self.head = Some(node)
        } else {
            unsafe {
                (*self.tail).next = Some(node);
            }
        }
        self.tail = node_ptr;
    }

    fn pop(&mut self) -> Option<T> {
        let node = self.head.take()?;
        self.head = node.next;
        if self.head.is_none() {
            self.tail = ptr::null_mut()
        }
        Some(node.elem)
    }
}

fn main() {}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);
    }
}
