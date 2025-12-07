use std::{env, error::Error};

mod process;

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = env::args().nth(1).expect("usage: elk FILE");

    let mut proc = process::Process::new();
    let _ = proc.load_object_and_dependencies(input_path)?;
    println!("{proc:#?}");

    Ok(())
}
