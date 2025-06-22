use std::{collections::HashMap, env, error::Error, fs, io};

use glob::{MatchOptions, glob_with};
use walkdir::{DirEntry, WalkDir};

fn modified() -> Result<(), io::Error> {
    let cur_dir = env::current_dir()?;
    println!("Entries modified in the last 24 hours in {:?}:", cur_dir);
    for entry in fs::read_dir(cur_dir)? {
        let path = entry?.path();

        let metadata = fs::metadata(&path)?;
        let last_modified = metadata
            .modified()?
            .elapsed()
            .expect("No modified")
            .as_secs();
        if last_modified < 24 * 60 * 60 && metadata.is_file() {
            println!(
                "Last modified: {:?} seconds, is read only: {:?}, size: {:?} bytes, filename: {:?}",
                last_modified,
                metadata.permissions().readonly(),
                metadata.len(),
                path.file_name().expect("No filename")
            );
        }
    }
    Ok(())
}

fn modified_jsons() -> Result<(), io::Error> {
    let cur_dir = env::current_dir()?;
    println!("Json modified in the last 24 hours in {:?}:", cur_dir);
    for entry in WalkDir::new(cur_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = String::from(entry.file_name().to_string_lossy());
        let metadata = entry.metadata()?;
        let last_modified = metadata
            .modified()?
            .elapsed()
            .expect("No modified")
            .as_secs();
        if name.ends_with(".json") && last_modified < 24 * 60 * 60 && metadata.is_file() {
            print!("{}, ", name);
        }
    }
    println!();
    Ok(())
}

fn find_duplicates() {
    println!("Dublicates: ");
    let mut filenames = HashMap::new();
    let mut max_dup_count = 5;
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let name = String::from(entry.file_name().to_string_lossy());
        let counter = filenames.entry(name.clone()).or_insert(0);
        *counter += 1;
        if *counter == 2 {
            max_dup_count -= 1;
            println!("{}", name);
        }
        if max_dup_count == 0 {
            break;
        }
    }
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

fn find_unhidden() {
    WalkDir::new(".")
        .min_depth(2)
        .max_depth(4)
        .into_iter()
        .filter_entry(|e| is_not_hidden(e))
        .filter_map(|r| r.ok())
        .take(5)
        .for_each(|x| println!("{}", x.path().display()));
}

fn find_pngs() -> Result<(), Box<dyn Error>> {
    let options = MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };
    for entry in glob_with("**/*.png", options)? {
        println!("{}", entry?.display());
    }
    Ok(())
}

fn main() {
    modified().unwrap();
    modified_jsons().unwrap();
    find_duplicates();
    find_unhidden();
    find_pngs().unwrap();
}
