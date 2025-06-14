use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    static ref PRIVILEGES: HashMap<&'static str, Vec<&'static str>> = {
        println!("fill map");
        let mut m = HashMap::new();
        m.insert("James", vec!["user", "admin"]);
        m.insert("Jim", vec!["user"]);
        m
    };
}

fn show_access(name: &str) {
    let access = PRIVILEGES.get(name);
    println!("{name}: {access:?}");
}

fn main() {
    println!("start program");
    show_access("Jim");

    let access = PRIVILEGES.get("James");
    println!("James: {access:?}");
}
