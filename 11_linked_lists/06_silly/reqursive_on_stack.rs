//The Stack-Allocated Linked List

pub struct List<'a, T> {
    pub data: T,
    pub prev: Option<&'a List<'a, T>>,
}

impl<'a, T> List<'a, T> {
    pub fn push<U>(
        prev: Option<&'a List<'a, T>>,
        data: T,
        callback: impl FnOnce(&List<'a, T>) -> U,
    ) -> U {
        let list = Self { data, prev };
        callback(&list)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter { next: Some(self) }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a List<'a, T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next?;
        self.next = node.prev;
        Some(&node.data)
    }
}

fn main() {
    List::push(None, 3, |list| {
        println!("{}", list.data);
        List::push(Some(list), 5, |list| {
            println!("{}", list.data);
            List::push(Some(list), 13, |list| {
                println!("{}", list.data);
            })
        })
    })
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn elegance() {
        List::push(None, 3, |list| {
            assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![3]);
            List::push(Some(list), 5, |list| {
                assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![5, 3]);
                List::push(Some(list), 13, |list| {
                    assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![13, 5, 3]);
                })
            })
        })
    }

    #[test]
    fn cell() {
        use std::cell::Cell;

        List::push(None, Cell::new(3), |list| {
            List::push(Some(list), Cell::new(5), |list| {
                List::push(Some(list), Cell::new(13), |list| {
                    for val in list.iter() {
                        val.set(val.get() * 10)
                    }
                    assert_eq!(
                        list.iter().map(|cell| cell.get()).collect::<Vec<_>>(),
                        vec![130, 50, 30]
                    );
                })
            })
        })
    }
}
