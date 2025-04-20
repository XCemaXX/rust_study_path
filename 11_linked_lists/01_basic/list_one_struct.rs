#[derive(Debug)]
enum List<T> {
    Element(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn push(self, val: T) -> Self {
        Self::Element(val, Box::new(self))
    }
    fn top(&self) -> Option<&T> {
        match self {
            Self::Element(value, _) => Some(value),
            Self::Nil => None,
        }
    }
    fn pop(self) -> Self {
        match self {
            Self::Element(_, tail) => *tail,
            Self::Nil => self,
        }
    }
    fn get_n(&self, n: usize) -> Option<&T> {
        let mut n = n;
        let mut node = self;
        while let Self::Element(_, ref tail) = *node {
            if n == 0 {
                break;
            }
            node = tail;
            n -= 1;
        }
        match node {
            Self::Element(value, _) => Some(value),
            Self::Nil => None,
        }
    }
    fn len(&self) -> usize {
        let mut n = 0_usize;
        let mut node = self;
        while let Self::Element(_, ref tail) = *node {
            node = tail;
            n += 1;
        }
        n
    }
}

impl<T> std::fmt::Display for List<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        let mut node = self;
        while let Self::Element(ref value, ref tail) = *node {
            match **tail {
                Self::Element(_, _) => write!(f, "{}, ", value)?,
                Self::Nil => write!(f, "{}", value)?,
            };
            node = tail;
        }
        write!(f, "]")
    }
}

mod test {
    use super::*;
    #[test]
    fn test1() {
        let mut list: List<i32> =
            List::Element(23, Box::new(List::Element(42, Box::new(List::Nil))));
        list = List::Element(1, Box::new(list));
        list = list.push(2);
        assert_eq!(4, list.len());
        println!("Debug view: {:?}; Len: {}", list, list.len());
        list = list.pop();
        println!("fmt::Display view: {}. Len: {}", list, list.len());
        println!("Top: {:?}; 3d: {:?}", list.top(), list.get_n(2));
        assert_eq!(3, list.len());
        assert_eq!(list.top(), Some(&1));
        assert_eq!(list.get_n(2), Some(&42));
    }
}
