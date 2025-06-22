use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Error, Write};

use memmap::Mmap;
use same_file::Handle;

const PATH: &str = "/tmp/files.txt";

fn simple() -> Result<(), Error> {
    let mut output = File::create(PATH)?;
    write!(output, "My hovercraft is full of eels!")?;

    let input = File::open(PATH)?;
    let buf = BufReader::new(input);

    for line in buf.lines() {
        println!("{}", line?);
    }
    Ok(())
}

fn file_content_to_upper(dst: &str) -> Result<(), Error> {
    let read_handle = Handle::from_path(PATH)?;
    let write_handle = if let Ok(write_handle) = Handle::from_path(dst) {
        write_handle
    } else {
        let _ = File::create(dst)?;
        Handle::from_path(dst)?
    };
    if write_handle == read_handle {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Same file for read and write",
        ))
    } else {
        drop(write_handle);
        let file = File::open(PATH)?;
        let file = BufReader::new(file);
        let mut output = OpenOptions::new().write(true).open(dst)?;
        for (num, line) in file.lines().enumerate() {
            write!(output, "{}: {}", num, line?.to_uppercase())?;
        }
        Ok(())
    }
}

fn mmap_file() -> Result<(), Error> {
    let file = File::open(PATH)?;
    let map = unsafe { Mmap::map(&file)? };

    let random_indexes = [0, 1, 2, 19, 22, 10, 11, 29];
    assert_eq!(&map[3..13], b"hovercraft");
    let random_bytes = random_indexes.iter().map(|&i| map[i]).collect::<Vec<_>>();
    assert_eq!(&random_bytes[..], b"My loaf!");
    Ok(())
}

fn main() {
    simple().unwrap();
    file_content_to_upper(PATH).expect_err("Should fail");
    file_content_to_upper("/tmp/files2.txt").unwrap();
    mmap_file().unwrap();
}
