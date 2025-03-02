use std::collections::HashMap;
use std::cell::Cell;
use std::cell::RefCell;

fn main() {
    // RefCell without copy
    let mut map: HashMap<String, String> = HashMap::new(); // or make straight away: HashMap<String, RefCell<String>>
    map.insert("a".to_owned(), "1".to_owned());
    map.insert("b".to_owned(), "2".to_owned());
    map.insert("c".to_owned(), "3".to_owned());
    let mut map: HashMap<String, RefCell<String>> = map.into_iter()
        .map(|(key, value)| (key, RefCell::new(value)))
        .collect();
    map.insert("e".to_owned(), "4".to_string().into());

    for (key, value) in map.iter() {
        value.borrow_mut().push_str("!");

        if let Some(other) = map.get("c") {
            println!("Processing key{:?}: value 'c' is {}", key, other.borrow());
        }
    }
    println!("Result: {:#?}", map);

    // Cell with copy
    let mut map: HashMap<String, Cell<i32>> = HashMap::new();
    map.insert("a".to_owned(), Cell::new(1));
    map.insert("b".to_owned(), Cell::new(2));
    map.insert("c".to_owned(), Cell::new(3));

    for (key, value) in map.iter() {
        value.set(value.get() + 10);

        if let Some(other) = map.get("a") {
            println!("Processing key{:?}: value 'a' is {}", key, other.get());
        }
    }
    println!("Result: {:#?}", map);

    // Copy of keys, raw value type
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("a".to_owned(), "Hello".to_owned());
    map.insert("b".to_owned(), "World".to_owned());
    map.insert("c".to_owned(), "Rust".to_owned());

    let keys: Vec<String> = map.keys().cloned().collect();

    for key in &keys {
        if let Some(value) = map.get_mut(key) {
            value.push_str(" updated");
        }

        if let Some(other) = map.get("a") {
            println!("Processing key{}: value 'a' is {}", key, other);
        }
    }

    println!("Result: {:#?}", map);

    // Unsafe with key pointers copy, raw value type
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("a".to_owned(), "Hello".to_owned());
    map.insert("b".to_owned(), "World".to_owned());
    map.insert("c".to_owned(), "Rust".to_owned());

    let keys: Vec<*const String> = map.keys().map(|k| k as *const String).collect();

    for &key_ptr in &keys {
        let key_ref = unsafe { &*key_ptr };

        if let Some(value) = map.get_mut(key_ref) {
            value.push_str(" updated");
        }

        if let Some(other) = map.get("a") {
            println!("Processing key{}: value 'a' is {}", key_ref, other);
        }
    }

    println!("Result: {:#?}", map);
}

