use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
enum CustomError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("got an invalid code {0}")]
    InvalidCode(u32),
}

fn read_code(file: &str) -> Result<u32, CustomError> {
    let _ = std::fs::read_to_string(file)?;
    Err(CustomError::InvalidCode(42))
}

fn parse_first_number(number: &str) -> Result<u64> {
    let number = number
        .split('.')
        .next()
        .context("cannot parse uptime data")?;
    Ok(number.parse()?)
}

fn main() -> Result<()> {
    for number in ["123.42", "3.6.4", "wrong"] {
        print!("str_number '{}': ", number);
        match parse_first_number(number) {
            Ok(number) => println!("{}", number),
            Err(err) => println!("{}", err),
        };
    }

    match read_code("fake_file") {
        Ok(code) => println!("Code: {}", code),
        Err(e) => println!("{}", e),
    };
    let res = match read_code("12_cookbook/11_error_handling/errors.rs") {
        Err(CustomError::IoError(_)) => unreachable!(),
        res => res,
    };
    let _valid_code = res.context("Failed to get a valid code from 'read_code'")?;

    Ok(())
}
