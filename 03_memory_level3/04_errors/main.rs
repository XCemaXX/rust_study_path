use std::panic;
use std::io::{self, Read};
use std::fs;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use anyhow::{bail, Context, Result};

fn read_username(path: &str) -> Result<String, io::Error> {
    let mut username_file = fs::File::open(path)?;
    let mut username = String::new();
    username_file.read_to_string(&mut username)?;
    Ok(username)
}
//############

#[derive(Debug)]
enum ReadUserNameError {
    IoError(io::Error),
    EmptyName(String),
}

impl Error for ReadUserNameError {}

impl Display for ReadUserNameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { 
        match self {
            Self::IoError(e) => write!(f, "IO error {e}"),
            Self::EmptyName(path) => write!(f, "Cannot get user name from {path}"),
        }
    }
}

impl From<io::Error> for ReadUserNameError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

fn read_username2(path: &str) -> Result<String, ReadUserNameError> {
    let mut username = String::with_capacity(100);
    fs::File::open(path)?.read_to_string(&mut username)?;
    if username.is_empty() {
        Err(ReadUserNameError::EmptyName(path.to_string()))
    } else {
        Ok(username)
    }
}
//#############
//any type of error can be returned
fn read_count(path: &str) -> Result<i32, Box<dyn Error>> {
    let mut count_str = String::new();
    fs::File::open(path)?.read_to_string(&mut count_str)?; //io error
    let count: i32 = count_str.parse()?; //std::num::ParseIntError from String::parse
    Ok(count)
}
//##############
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[error("Cannot get user name from  {0}")]
struct ReadUserNameError2(String);

fn read_username3(path: &str) -> Result<String> {
    let mut username = String::with_capacity(100);
    fs::File::open(path)
        .with_context(|| format!("Cannot open file {path}"))?
        .read_to_string(&mut username)
        .context("Read file error")?;
    if username.is_empty() {
        bail!(ReadUserNameError2(path.to_string()));
    }
    Ok(username)
}


fn main() {
    let result = panic::catch_unwind(|| "No problem here!");
    println!("{result:?}");

    let result = panic::catch_unwind(|| {
        panic!("Oh no!");
    });
    println!("{result:?}");

    let username = read_username("fake.file");
    println!("username1 or error: {username:?}");
    let username = read_username2("fake.file");
    println!("username2 or error: {username:?}");
    let username = read_username3("fake.file");
    println!("username3 or error: {username:?}");

    //fs::write("count.dat", "1i3").unwrap();
    match read_count("count.dat") {
        Ok(count) => println!("Content: {count}"),
        Err(err) => println!("Error: {err}"),
    };
}