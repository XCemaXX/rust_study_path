use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Result};

/// Description of Option type
///
/// Example of documentation
fn option_type() {
    let name = "Löwe 老虎 Léopard Gepardi";
    let mut position: Option<usize> = name.find('é');
    println!("Position {position:?}");
    assert_eq!(position.unwrap(), 14);
    position = name.find('Z');
    println!("Position {position:?}");
    // assert_eq!(position.expect("Not found panic"), 0);
    // unwrap()/expect() extracts value from Option
}

fn result_type1(file_name: &str) -> std::result::Result<(String, usize), String> {
    let file = File::open(file_name);
    match file {
        Ok(mut file) => {
            let mut buf = String::new();
            if let Ok(len) = file.read_to_string(&mut buf) {
                Ok((buf, len))
            } else {
                Err(String::from("Cannot read {file_name}"))
            }
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

fn result_type2(file_name: &str) -> Result<()> {
    let mut file = File::open(file_name)?;
    let mut buf = String::new();
    let len = file.read_to_string(&mut buf)?;
    println!(" File content: {buf}\n ({len} bytes)");
    Ok(())
    // unwrap()/expect() extracts value from Result
}

fn string_type() {
    let mut s1 = String::new();
    s1.push_str("String");
    println!("s1: len = {}, cap = {}", s1.len(), s1.capacity());

    let mut s2 = String::with_capacity(s1.capacity() + 10);
    s2.push_str(&s1); 
    s2.push('2');
    println!("s2: len = {}, cap = {}", s2.len(), s2.capacity());

    let s3 = String::from("🇨🇭");
    println!("s3: len = {}, syms = {}", s3.len(), s3.chars().count());

    // For String: & = Deref<Target = str>
    //let s4 = s1.deref(); // &str
    //let s5 = &*s1; // &str
}

fn vec_type() {
    let mut v1 = Vec::new();
    v1.push(23);
    println!("v1: len = {}, cap = {}", v1.len(), v1.capacity());

    let mut v2 = Vec::with_capacity(v1.capacity() + 10);
    v2.extend(v1.iter());
    v2.push(42);
    println!("v2: len = {}, cap = {}", v2.len(), v2.capacity());

    let mut v3 = vec![0, 0, 1, 2, 3, 4, 5];
    v3.retain(|x| x % 2 == 0); // filter. Only % 2 == 1 will be removed
    println!("v3: {v3:?}");
    v3.dedup(); // remove seq duplicates
    println!("v3: {v3:?}");
    //get elem: [], .get()
}

fn hash_map_type() {
    let mut books = HashMap::new();
    books.insert("Adventures of Huckleberry Finn".to_string(), 207);
    books.insert(String::from("Grimms' Fairy Tales"), 751);
    books.insert("Pride and Prejudice".to_string(), 303);
    
    let _ = HashMap::from([
        ("Harry Potter and the Sorcerer's Stone".to_string(), 336),
        ("The Hunger Games".to_string(), 374),
      ]);

    let pname = "Prostokvashino";
    if !books.contains_key(pname) {
        println!("We know about {} books, but not {pname}.", books.len());
    }

    for book in ["Pride and Prejudice", pname] {
        match books.get(book) {
            Some(page_count) => println!("{book}: {page_count} pages"),
            None => println!("{book} is unknown."),
        }
    }
    println!("{books:#?}");
    for book in ["Pride and Prejudice", pname, "Alice's Adventure in Wonderland"] {
        // if not found key, then insert it with default value
        let page_count: &mut i32 = books.entry(book.to_string()).or_insert(2);
        // change the value in the map
        *page_count *= 10000;
    }
    println!("{books:#?}");

    let _ = books.get("Harry Potter and the Sorcerer's Stone").unwrap_or(&336); // not change map, returns val if not found
}

fn main() -> Result<()> {
    option_type();
    string_type();
    vec_type();
    hash_map_type();

    let mut res1 = result_type1("02_basics_level2/04_std_types/test.txt");
    println!("{res1:?}");
    res1 = result_type1("fake.txt");
    println!("{res1:?}");

    let res = result_type2("02_basics_level2/04_std_types/test.txt");
    println!("{res:?}");
    result_type2("fake.txt")
}