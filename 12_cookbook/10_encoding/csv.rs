#![allow(dead_code)]

use csv::{Error, Reader, Writer};
use serde::{Deserialize, Serialize, de};
use std::{
    io::{self, ErrorKind},
    str::FromStr,
};

#[derive(Deserialize, Debug)]
struct Car {
    year: u16,
    make: String,
    model: String,
    description: String,
}

fn read_csv() -> Result<(), Error> {
    let csv = "year,make,model,description\n\
        1948,Porsche,356,Luxury sports car\n\
        1967,Ford,Mustang fastback 1967,American car";

    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    for record in reader.records() {
        let record = record?;
        println!("Weakly typed: {:?}", record);
    }
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    for record in reader.deserialize() {
        let record: Car = record?;
        println!("Strongly typed: {:?}", record);
    }
    Ok(())
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct City {
    City: String,
    State: String,
    #[serde(deserialize_with = " csv::invalid_option")]
    Population: Option<u64>,
    Latitude: f64,
    Longitude: f64,
}

fn filter_records() -> Result<(), Error> {
    println!("---Filer");
    let query = "CA";
    let csv = "\
        City\tState\tPopulation\tLatitude\tLongitude\n\
        Kenai\tAK\t7610\t60.5544444\t-151.2583333\n\
        Oakman\tAL\t\t33.7133333\t-87.3886111\n\
        Sandfort\tAL\t\t32.3380556\t-85.2233333\n\
        West Hollywood\tCA\t37031\t34.0900000\t-118.3608333";
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(csv.as_bytes());
    let mut writer = csv::Writer::from_writer(io::stdout());

    writer.write_record(reader.headers()?)?;
    for record in reader.records() {
        let record = record?;
        if record.iter().any(|field| field == query) {
            writer.write_record(&record)?;
        }
    }
    writer.flush()?;
    println!("---Handle empty fields");
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(csv.as_bytes());
    for record in reader.deserialize::<City>() {
        println!("{:?}", record?);
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct Man<'a> {
    name: &'a str,
    place: &'a str,
    id: u64,
}

fn serialize() -> Result<(), Error> {
    println!("---Serialize");
    let mut writer = csv::Writer::from_writer(io::stdout());
    writer.write_record(&["Name", "Place", "ID"])?;
    writer.serialize(("Bob", "Sydney", 87))?;
    writer.serialize(("Alex", "Delhi", 11))?;
    writer.flush()?;

    writer = csv::Writer::from_writer(io::stdout());

    let records = [
        Man {
            name: "Mark",
            place: "Melbourne",
            id: 56,
        },
        Man {
            name: "Ashley",
            place: "Sydney",
            id: 64,
        },
    ];

    records
        .iter()
        .for_each(|man| writer.serialize(man).unwrap());
    writer.flush()?;

    Ok(())
}

#[derive(Debug)]
struct HexColor {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug, Deserialize)]
struct Row {
    color_name: String,
    color: HexColor,
}

impl FromStr for HexColor {
    type Err = io::Error;

    fn from_str(hex_color: &str) -> Result<Self, Self::Err> {
        let trimmed = hex_color.trim_matches('#');
        if trimmed.len() != 6 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "hex must be 6 digits",
            ));
        }
        let parse_hex = |s| {
            u8::from_str_radix(s, 16).map_err(|_| io::Error::new(ErrorKind::InvalidData, "bad hex"))
        };
        Ok(HexColor {
            red: parse_hex(&trimmed[..2])?,
            green: parse_hex(&trimmed[2..4])?,
            blue: parse_hex(&trimmed[4..6])?,
        })
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<HexColor, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        HexColor::from_str(s).map_err(de::Error::custom)
    }
}

fn transform_colors() -> Result<(), Box<dyn std::error::Error>> {
    println!("---Colors");
    let data = "color_name,color\n\
        red,#ff0000\n\
        green,#00ff00\n\
        blue,#0000FF\n\
        periwinkle,#ccccff\n\
        magenta,#ff00ff";
    let mut out = Writer::from_writer(vec![]);
    let mut reader = Reader::from_reader(data.as_bytes());

    for row in reader.deserialize::<Row>() {
        let row = row?;
        out.serialize((
            row.color_name,
            row.color.red,
            row.color.green,
            row.color.blue,
        ))?;
    }
    let written = String::from_utf8(out.into_inner()?)?;
    assert_eq!(Some("magenta,255,0,255"), written.lines().last());
    println!("{}", written);
    Ok(())
}

fn main() {
    read_csv().unwrap();
    filter_records().unwrap();
    serialize().unwrap();
    transform_colors().unwrap();
}
