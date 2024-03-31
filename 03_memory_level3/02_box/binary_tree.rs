#[derive(Debug)]
struct Node<T: Ord> {
    value: T,
    left: Subtree<T>,
    right: Subtree<T>,
}

#[derive(Debug)]
struct Subtree<T: Ord>(Option<Box<Node<T>>>); //can be empty

#[derive(Debug)]
pub struct BinaryTree<T: Ord> { //only unique values
    root: Subtree<T>,
}

impl<T: Ord> BinaryTree<T> {
    fn new() -> Self {
        Self{root: Subtree::new()}
    }

    fn insert(&mut self, value: T) {
        self.root.insert(value);
    }

    fn has(&self, value: &T) -> bool {
        self.root.has(value)
    }

    fn len(&self) -> usize {
        self.root.len()
    }

    fn iter(&self) -> BinaryTreeIter<T> {
        self.root.iter()
    }
}

impl<T: Ord> Subtree<T> {
    fn new() -> Self {
        Self(None)
    }

    fn insert(&mut self, value: T) {
        let root = &mut self.0;
        match root {
            None => *root = Some(Box::new(Node::new(value))),
            Some(b) => match value.cmp(&b.value) {
                std::cmp::Ordering::Less => b.left.insert(value),
                std::cmp::Ordering::Equal => {},
                std::cmp::Ordering::Greater => b.right.insert(value),
            }
        }
    }

    fn has(&self, value: &T) -> bool {
        match &self.0 {
            None => false,
            Some(b) => match value.cmp(&b.value) {
                std::cmp::Ordering::Less => b.left.has(value),
                std::cmp::Ordering::Equal => true,
                std::cmp::Ordering::Greater => b.right.has(value),
            }
        }
    }

    fn len(&self) -> usize {
        match &self.0 {
            None => 0,
            Some(b) => 1 + b.left.len() + b.right.len(), 
        }
    }

    fn iter(&self) -> BinaryTreeIter<T> {
        match &self.0 {
            None => BinaryTreeIter {parents: vec![]},
            Some(b) => {
                BinaryTreeIter {parents: vec![b]}
            }
        }
        
    }
}

impl<T: Ord> Node<T> {
    fn new(value: T) -> Self {
        Self{value: value, left: Subtree::new(), right: Subtree::new()}
    }
}

struct BinaryTreeIter<'a, T: Ord> {
    parents: Vec<&'a Node<T>>
}

impl <'a, T: Ord>Iterator for BinaryTreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.parents.len() == 0 {
            return None
        };
        let root = &self.parents.pop();
        match root {
            None => None,
            Some(b) => {
                let res = Some(&b.value);
                if let Some(next) = &b.right.0 {
                    self.parents.push(&next);
                }
                if let Some(next) = &b.left.0 {
                    self.parents.push(&next);
                }
                res
            },
        }   
    }
}


fn main() {
    let mut tree = BinaryTree::new();
    tree.insert("foo");
    assert_eq!(tree.len(), 1);
    tree.insert("bar");
    assert!(tree.has(&"foo"));

    let mut tree2 = BinaryTree::new();
    tree2.insert(5);
    tree2.insert(10);
    tree2.insert(1);
    tree2.insert(3);
    for i in tree2.iter() {
        println!("{}",i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len() {
        let mut tree = BinaryTree::new();
        assert_eq!(tree.len(), 0);
        tree.insert(2);
        assert_eq!(tree.len(), 1);
        tree.insert(1);
        assert_eq!(tree.len(), 2);
        tree.insert(2); // дубликат
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn has() {
        let mut tree = BinaryTree::new();
        fn check_has(tree: &BinaryTree<i32>, exp: &[bool]) {
            let got: Vec<bool> =
                (0..exp.len()).map(|i| tree.has(&(i as i32))).collect();
            assert_eq!(&got, exp);
        }

        check_has(&tree, &[false, false, false, false, false]);
        tree.insert(0);
        check_has(&tree, &[true, false, false, false, false]);
        tree.insert(4);
        check_has(&tree, &[true, false, false, false, true]);
        tree.insert(4);
        check_has(&tree, &[true, false, false, false, true]);
        tree.insert(3);
        check_has(&tree, &[true, false, false, true, true]);
    }

    #[test]
    fn unbalanced() {
        let mut tree = BinaryTree::new();
        for i in 0..100 {
            tree.insert(i);
        }
        assert_eq!(tree.len(), 100);
        assert!(tree.has(&50));
    }
}