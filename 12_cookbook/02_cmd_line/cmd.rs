#![allow(dead_code)]

use std::path::PathBuf;

use clap::{Arg, Command, builder::PathBufValueParser};

fn main() {
    // cargo run --bin cook_cmd_line -- -f 1.txt -n 23
    let args = Command::new("Cmd line")
        .version("1.0.0")
        .about("Help for cmd line")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Input file to read")
                .value_parser(PathBufValueParser::default()),
        )
        .arg(
            Arg::new("num")
                .short('n')
                .long("number")
                .help("Some number"),
        )
        .get_matches();
    let default_file = PathBuf::from("main.rs");
    let file = args.get_one("file").unwrap_or(&default_file);
    println!("The file passed is: {}", file.display());

    let num_str = args.get_one::<String>("num");
    match num_str {
        None => println!("No number T_T"),
        Some(s) => match s.parse::<i32>() {
            Ok(n) => println!("Number is: {}", n),
            Err(_) => println!("That's not a number: '{}'", s),
        },
    }
}
