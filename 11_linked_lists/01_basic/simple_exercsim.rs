// solution for https://exercism.org/tracks/rust/exercises/simple-linked-list

struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}
pub struct SimpleLinkedList<T> {
    len: usize,
    head: Option<Box<Node<T>>>,
}

impl<T> SimpleLinkedList<T> {
    pub fn new() -> Self {
        Self { len: 0, head: None }
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn push(&mut self, element: T) {
        self.len += 1;
        self.head = Some(Box::new(Node {
            data: element,
            next: self.head.take(),
        }));
    }
    pub fn pop(&mut self) -> Option<T> {
        let node = self.head.take()?;
        self.len -= 1;
        self.head = node.next;
        Some(node.data)
    }
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.data)
    }
    #[must_use]
    pub fn rev(mut self) -> SimpleLinkedList<T> {
        let mut list = Self::new();
        while let Some(data) = self.pop() {
            list.push(data);
        }
        list
    }
}
impl<T> FromIterator<T> for SimpleLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self { len: 0, head: None };
        iter.into_iter().for_each(|data| list.push(data));
        list
    }
}
impl<T> From<SimpleLinkedList<T>> for Vec<T> {
    fn from(mut linked_list: SimpleLinkedList<T>) -> Vec<T> {
        let mut v = Vec::from_iter((0..linked_list.len()).map(|_| linked_list.pop().unwrap()));
        v.reverse();
        v
    }
}

mod test {
    use super::*;
    #[test]
    fn new_list_is_empty() {
        let list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        assert_eq!(list.len(), 0, "list's length must be 0");
    }
    #[test]
    fn push_increments_length() {
        let mut list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        list.push(1);
        assert_eq!(list.len(), 1, "list's length must be 1");
        list.push(2);
        assert_eq!(list.len(), 2, "list's length must be 2");
    }
    #[test]
    fn pop_decrements_length() {
        let mut list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        list.push(1);
        list.push(2);
        list.pop();
        assert_eq!(list.len(), 1, "list's length must be 1");
        list.pop();
        assert_eq!(list.len(), 0, "list's length must be 0");
    }
    #[test]
    fn is_empty() {
        let mut list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        assert!(list.is_empty(), "List wasn't empty on creation");
        for inserts in 0..100 {
            for i in 0..inserts {
                list.push(i);
                assert!(
                    !list.is_empty(),
                    "List was empty after having inserted {i}/{inserts} elements"
                );
            }
            for i in 0..inserts {
                assert!(
                    !list.is_empty(),
                    "List was empty before removing {i}/{inserts} elements"
                );
                list.pop();
            }
            assert!(
                list.is_empty(),
                "List wasn't empty after having removed {inserts} elements"
            );
        }
    }
    #[test]
    fn pop_returns_head_element_and_removes_it() {
        let mut list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        list.push(1);
        list.push(2);
        assert_eq!(list.pop(), Some(2), "Element must be 2");
        assert_eq!(list.pop(), Some(1), "Element must be 1");
        assert_eq!(list.pop(), None, "No element should be contained in list");
    }
    #[test]
    fn peek_returns_reference_to_head_element_but_does_not_remove_it() {
        let mut list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        assert_eq!(list.peek(), None, "No element should be contained in list");
        list.push(2);
        assert_eq!(list.peek(), Some(&2), "Element must be 2");
        assert_eq!(list.peek(), Some(&2), "Element must be still 2");
        list.push(3);
        assert_eq!(list.peek(), Some(&3), "Head element is now 3");
        assert_eq!(list.pop(), Some(3), "Element must be 3");
        assert_eq!(list.peek(), Some(&2), "Head element is now 2");
        assert_eq!(list.pop(), Some(2), "Element must be 2");
        assert_eq!(list.peek(), None, "No element should be contained in list");
    }
    #[test]
    fn from_slice() {
        let mut array = vec!["1", "2", "3", "4"];
        let mut list: SimpleLinkedList<_> = array.drain(..).collect();
        assert_eq!(list.pop(), Some("4"));
        assert_eq!(list.pop(), Some("3"));
        assert_eq!(list.pop(), Some("2"));
        assert_eq!(list.pop(), Some("1"));
    }
    #[test]
    fn reverse() {
        let mut list: SimpleLinkedList<u32> = SimpleLinkedList::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut rev_list = list.rev();
        assert_eq!(rev_list.pop(), Some(1));
        assert_eq!(rev_list.pop(), Some(2));
        assert_eq!(rev_list.pop(), Some(3));
        assert_eq!(rev_list.pop(), None);
    }
    #[test]
    fn into_vector() {
        let mut v = Vec::new();
        let mut s = SimpleLinkedList::new();
        for i in 1..4 {
            v.push(i);
            s.push(i);
        }
        let s_as_vec: Vec<i32> = s.into();
        assert_eq!(v, s_as_vec);
    }
}
