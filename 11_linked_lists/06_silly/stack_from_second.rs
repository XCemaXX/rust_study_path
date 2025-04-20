use std::mem;

pub(super) struct Node<T> {
    elem: T,
    next: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

pub struct Stack<T> {
    head: Link<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let node = Box::new(Node {
            elem,
            next: None,
        });
        self.push_node(node);
    }

    pub(crate) fn push_node(&mut self, mut node: Box<Node<T>>) {
        node.next = self.head.take();
        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.pop_node().map(|node| node.elem)
    }

    pub(crate) fn pop_node(&mut self) -> Option<Box<Node<T>>> {
        let mut node = self.head.take()?;
        self.head = node.next.take();
        Some(node)
    }

    pub fn peek(&self) -> Option<&T> {
        let node = self.head.as_ref()?;
        Some(&node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = mem::replace(&mut node.next, None);
        }
    }
}
