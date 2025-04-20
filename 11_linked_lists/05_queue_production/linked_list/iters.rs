use super::*;

impl<T> LinkedList<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: PhantomData,
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    pub(super) front: Link<T>,
    pub(super) back: Link<T>,
    pub(super) len: usize,
    pub(super) _boo: PhantomData<&'a T>,
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let node = self.front?;
        let node = unsafe { node.as_ref() };
        self.len -= 1;
        self.front = node.back;
        Some(&node.elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let node: NonNull<Node<T>> = self.back?;
        let node = unsafe { node.as_ref() };
        self.len -= 1;
        self.back = node.front;
        Some(&node.elem)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.len
    }
}

////////////////////////// IntoIter
pub struct IntoIter<T> {
    pub(super) list: LinkedList<T>,
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

////////////////////////// IterMut
pub struct IterMut<'a, T> {
    pub(super) front: Link<T>,
    pub(super) back: Link<T>,
    pub(super) len: usize,
    pub(super) _boo: PhantomData<&'a mut T>,
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let mut node = self.front?;
        let node = unsafe { node.as_mut() };
        self.len -= 1;
        self.front = node.back;
        Some(&mut node.elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let mut node: NonNull<Node<T>> = self.back?;
        let node = unsafe { node.as_mut() };
        self.len -= 1;
        self.back = node.front;
        Some(&mut node.elem)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {
    fn len(&self) -> usize {
        self.len
    }
}
