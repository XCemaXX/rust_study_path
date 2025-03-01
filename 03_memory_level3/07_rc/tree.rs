use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
struct Tree<T> {
    value: T,
    children: Vec<Rc<RefCell<Tree<T>>>>,
}

impl<T> Tree<T>
where T: std::default::Default,
    T: std::iter::Sum,
    T: std::ops::Add<Output = T>,
    T: Copy {
    fn new(value: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Tree {value, ..Tree::default() }))
    }

    fn sum(&self) -> T {
        self.value + self.children.iter().map(|c| c.borrow().sum()).sum::<T>()
    }
}

fn main() {
    let root = Tree::new(1);
    root.borrow_mut().children.push(Tree::new(5));
    let subtree = Tree::new(10);
    subtree.borrow_mut().children.push(Tree::new(11));
    subtree.borrow_mut().children.push(Tree::new(12));
    root.borrow_mut().children.push(subtree);
    println!("tree: {:#?}", root);
    println!("tree sum: {}", root.borrow().sum());
}