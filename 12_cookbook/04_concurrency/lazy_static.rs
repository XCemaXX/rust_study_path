use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref FRUIT: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn insert(fruit: &str) -> Result<(), &str> {
    let mut data = FRUIT.lock().map_err(|_| "Failed to get mutex")?;
    data.push(fruit.to_owned());
    Ok(())
}

fn main() -> Result<(), &'static str> {
    insert("apple")?;
    insert("orange")?;
    insert("peach")?;
    {
        let data = FRUIT.lock().map_err(|_| "Failed to get mutex")?;
        data.iter().for_each(|f| print!("{} ", f));
    }
    insert("grape")?;
    println!();
    Ok(())
}
