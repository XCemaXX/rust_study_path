// doubly-linked list (safe, but bad without itermut) (Queue)

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            next: None,
            prev: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let node = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                node.borrow_mut().next = Some(old_head.clone());
                old_head.borrow_mut().prev = Some(node.clone());
                self.head = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let node = self.head.take()?;
        match node.borrow_mut().next.take() {
            Some(new_head) => {
                new_head.borrow_mut().prev.take();
                self.head = Some(new_head);
            }
            None => {
                self.tail.take();
            }
        }
        let node = Rc::try_unwrap(node).ok().unwrap().into_inner();
        Some(node.elem)
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        let node = self.head.as_ref()?;
        Some(Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        let node = self.head.as_ref()?;
        Some(RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    pub fn push_back(&mut self, elem: T) {
        let node = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                node.borrow_mut().prev = Some(old_tail.clone());
                old_tail.borrow_mut().next = Some(node.clone());
                self.tail = Some(node);
            }
            None => {
                self.tail = Some(node.clone());
                self.head = Some(node);
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let node = self.tail.take()?;
        match node.borrow_mut().prev.take() {
            Some(new_tail) => {
                new_tail.borrow_mut().next.take();
                self.tail = Some(new_tail);
            }
            None => {
                self.head.take();
            }
        }
        let node = Rc::try_unwrap(node).ok().unwrap().into_inner();
        Some(node.elem)
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        let node = self.tail.as_ref()?;
        Some(Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        let node = self.tail.as_ref()?;
        Some(RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics_front() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn peek_front() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(list.pop_front(), Some(3));
    }
    #[test]
    fn basics_back() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek_back() {
        let mut list = List::new();
        assert!(list.peek_back().is_none());
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(&*list.peek_back().unwrap(), &3);
        assert_eq!(list.pop_back(), Some(3));
    }

    #[test]
    fn peek2() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        {
            let mut e = list.peek_front_mut().unwrap();
            *e = 22;
        }
        {
            let mut e = list.peek_back_mut().unwrap();
            *e = 45;
        }
        assert_eq!(list.pop_front(), Some(22));
        assert_eq!(list.pop_back(), Some(45));
        {
            let mut e = list.peek_front_mut().unwrap();
            *e = 100;
        }
        {
            let mut e = list.peek_back_mut().unwrap();
            *e = 1000;
        }
        assert_eq!(&*list.peek_back().unwrap(), &1000);
        assert_eq!(list.pop_front(), Some(1000));
        assert!(list.pop_back().is_none());
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
