use super::*;

pub struct CursorMut<'a, T> {
    cur: Link<T>,
    list: &'a mut LinkedList<T>,
    index: Option<usize>,
}

impl<T> LinkedList<T> {
    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        CursorMut {
            cur: None,
            list: self,
            index: None,
        }
    }
}

impl<T> CursorMut<'_, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            self.cur = unsafe { cur.as_ref().back };
            match self.cur {
                Some(_) => *self.index.as_mut().unwrap() += 1,
                None => self.index = None,
            }
        } else if self.list.is_empty() {
            return;
        } else {
            self.index = Some(0);
            self.cur = self.list.front;
        }
    }

    pub fn move_prev(&mut self) {
        if let Some(cur) = self.cur {
            self.cur = unsafe { cur.as_ref().front };
            match self.cur {
                Some(_) => *self.index.as_mut().unwrap() -= 1,
                None => self.index = None,
            }
        } else if self.list.is_empty() {
            return;
        } else {
            self.index = Some(self.list.len - 1);
            self.cur = self.list.back;
        }
    }

    pub fn current(&mut self) -> Option<&mut T> {
        let mut node = self.cur?;
        unsafe { Some(&mut node.as_mut().elem) }
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            let next = if let Some(cur) = self.cur {
                cur.as_ref().back
            } else {
                self.list.front
            };
            next.map(|mut node| &mut node.as_mut().elem)
        }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            let prev = if let Some(cur) = self.cur {
                cur.as_ref().front
            } else {
                self.list.back
            };
            prev.map(|mut node| &mut node.as_mut().elem)
        }
    }

    pub fn split_before(&mut self) -> LinkedList<T> {
        // We have this:
        //     list.front -> A <-> B <-> C <-> D <- list.back
        //                               ^
        //                              cur
        // And we want to produce this:
        //     list.front -> C <-> D <- list.back
        //                   ^
        //                  cur
        //    return.front -> A <-> B <- return.back
        if self.cur.is_none() {
            return std::mem::replace(self.list, LinkedList::new());
        }
        let mut cur = self.cur.unwrap();
        let src_len = self.list.len;
        let src_index = self.index.unwrap();
        let prev = unsafe { cur.as_ref().front };

        let self_len = src_len - src_index;
        let self_front = self.cur;
        let self_back = self.list.back;
        let self_index = Some(0);

        let ret_len = src_len - self_len;
        let ret_front = self.list.front;
        let ret_back = prev;

        if let Some(mut prev) = prev {
            unsafe {
                cur.as_mut().front = None;
                prev.as_mut().back = None;
            }
        }

        self.list.len = self_len;
        self.list.front = self_front;
        self.list.back = self_back;
        self.index = self_index;

        LinkedList {
            front: ret_front,
            back: ret_back,
            len: ret_len,
            _boo: PhantomData,
        }
    }

    pub fn split_after(&mut self) -> LinkedList<T> {
        // We have this:
        //     list.front -> A <-> B <-> C <-> D <- list.back
        //                         ^
        //                        cur
        // And we want to produce this:
        //     list.front -> A <-> B <- list.back
        //                         ^
        //                        cur
        //    return.front -> C <-> D <- return.back
        if self.cur.is_none() {
            return std::mem::replace(self.list, LinkedList::new());
        }
        let mut cur = self.cur.unwrap();
        let src_len = self.list.len;
        let src_index = self.index.unwrap();
        let next = unsafe { cur.as_ref().back };

        let self_len = src_index + 1;
        let self_back = self.cur;
        let self_front = self.list.front;
        let self_index = Some(src_index);

        let ret_len = src_len - self_len;
        let ret_front = next;
        let ret_back = self.list.back;

        if let Some(mut next) = next {
            unsafe {
                cur.as_mut().back = None;
                next.as_mut().front = None;
            }
        }

        self.list.len = self_len;
        self.list.front = self_front;
        self.list.back = self_back;
        self.index = self_index;

        LinkedList {
            front: ret_front,
            back: ret_back,
            len: ret_len,
            _boo: PhantomData,
        }
    }

    pub fn splice_before(&mut self, mut input: LinkedList<T>) {
        // We have this:
        // input.front -> 1 <-> 2 <- input.back
        // list.front -> A <-> B <-> C <- list.back
        //                     ^
        //                    cur
        // Becoming this:
        // list.front -> A <-> 1 <-> 2 <-> B <-> C <- list.back
        //                                 ^
        //                                cur
        if input.is_empty() {
            return;
        }
        if self.list.is_empty() {
            *self.list = input;
            return;
        }

        let mut input_front = input.front.take().unwrap();
        let mut input_back = input.back.take().unwrap();
        match (self.cur, self.list.back, self.index) {
            (Some(cur), _, Some(0)) => {
                let mut front = cur;
                unsafe {
                    front.as_mut().front = Some(input_back);
                    input_back.as_mut().back = Some(front);
                }
                self.list.front = Some(input_front);
                *self.index.as_mut().unwrap() += input.len;
            }
            (None, Some(mut back), _) => {
                unsafe {
                    back.as_mut().back = Some(input_front);
                    input_front.as_mut().front = Some(back);
                }
                self.list.back = Some(input_back);
            }
            (Some(mut cur), _, _) => {
                unsafe {
                    let mut prev = cur.as_ref().front.unwrap();
                    prev.as_mut().back = Some(input_front);
                    input_front.as_mut().front = Some(prev);
                    cur.as_mut().front = Some(input_back);
                    input_back.as_mut().back = Some(cur);
                }
                *self.index.as_mut().unwrap() += input.len;
            }
            _ => {
                panic!("Impossible cases")
            }
        }
        self.list.len += input.len;
        input.len = 0;
    }

    pub fn splice_after(&mut self, mut input: LinkedList<T>) {
        // We have this:
        // input.front -> 1 <-> 2 <- input.back
        // list.front -> A <-> B <-> C <- list.back
        //                     ^
        //                    cur
        // Becoming this:
        // list.front -> A <-> B <-> 1 <-> 2 <-> C <- list.back
        //                     ^
        //                    cur
        if input.is_empty() {
            return;
        }
        if self.list.is_empty() {
            *self.list = input;
            return;
        }
        let mut input_front = input.front.take().unwrap();
        let mut input_back = input.back.take().unwrap();
        match (self.cur, self.list.front, self.index) {
            (Some(cur), _, Some(index)) if index == self.list.len => {
                let mut back = cur;
                unsafe {
                    back.as_mut().back = Some(input_front);
                    input_front.as_mut().front = Some(back);
                }
                self.list.back = Some(input_back);
            }
            (None, Some(mut front), _) => {
                unsafe {
                    front.as_mut().front = Some(input_back);
                    input_back.as_mut().back = Some(front);
                }
                self.list.front = Some(input_front);
            }
            (Some(mut cur), _, _) => unsafe {
                let mut next = cur.as_ref().back.unwrap();
                next.as_mut().front = Some(input_back);
                input_back.as_mut().back = Some(next);
                cur.as_mut().back = Some(input_front);
                input_front.as_mut().front = Some(cur);
            },
            _ => {
                panic!("Impossible cases")
            }
        }
        self.list.len += input.len;
        input.len = 0;
    }

    pub fn remove_current(&mut self) -> Option<T> {
        let cur = self.cur?;
        unsafe {
            let node = cur.as_ref();
            let prev = node.front;
            let next = node.back;

            if let Some(mut p) = prev {
                p.as_mut().back = next;
            } else {
                self.list.front = next;
            }

            if let Some(mut n) = next {
                n.as_mut().front = prev;
            } else {
                self.list.back = prev;
            }

            let elem = std::ptr::read(&node.elem);
            let _ = Box::from_raw(cur.as_ptr());

            self.list.len -= 1;

            if let Some(n) = next {
                self.cur = Some(n);
            } else {
                self.cur = None;
                self.index = None;
            }
            Some(elem)
        }
    }

    pub fn insert_before(&mut self, elem: T) {
        let list = std::iter::once(elem).collect::<LinkedList<_>>();
        self.splice_before(list);
    }

    pub fn insert_after(&mut self, elem: T) {
        let list = std::iter::once(elem).collect::<LinkedList<_>>();
        self.splice_after(list);
    }
}
