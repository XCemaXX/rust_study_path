use std::{cell::RefCell, fmt::Display, ops::Deref, rc::Rc};

fn call(a: impl Deref + Display) {
    println!("{a}");
}

fn main() {
    let hello = Rc::new("hello".to_string());
    let hello_copy = Rc::clone(&hello);
    call(hello);
    println!("String exist: {hello_copy}");
    call(hello_copy);
    println!("String died");

    let hello = Rc::new(RefCell::new("hello2".to_string()));
    let hello_copy = Rc::clone(&hello);
    println!("Before change: {}", hello_copy.borrow());
    hello.borrow_mut().push_str(" world");
    println!("After change: {}", hello_copy.borrow());
}