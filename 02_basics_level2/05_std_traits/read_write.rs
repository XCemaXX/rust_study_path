use std::io::{Read, Write, BufRead, BufReader, Result};

fn count_lines<T: Read>(src: T) -> usize {
    let buf_reader = BufReader::new(src);
    buf_reader.lines().count()
}

fn read_example() -> Result <()> {
    let slice: &[u8] = b"foo\nbar\nbaz\n";
    println!("Lines in slice {}", count_lines(slice));

    let file = std::fs::File::open(std::env::current_exe()?)?;
    println!("Lines in file: {}", count_lines(file));

    Ok(())
}

fn log<T: Write>(writer: &mut T, msg: &str) -> Result <()> {
    writer.write_all(msg.as_bytes())?;
    writer.write_all("\n".as_bytes())
}

fn write_example() -> Result <String> {
    let mut buffer = Vec::new();
    log(&mut buffer, "hello")?;
    log(&mut buffer, "world")?;
    Ok(String::from_utf8(buffer).unwrap())
}

fn main()  {
    println!("{:?}", read_example());    
    println!("{:?}", write_example());    
}