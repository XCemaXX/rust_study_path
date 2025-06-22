#![allow(dead_code)]

mod json {
    use serde_json::json;

    pub fn parse_json() -> Result<(), serde_json::Error> {
        let s = r#"{
                 "userid": 103609,
                 "verified": true,
                 "access_privileges": [
                   "user",
                   "admin"
                 ]
               }"#;
        let parsed: serde_json::Value = serde_json::from_str(s)?;
        let expected = json!({
            "userid": 103609,
            "verified": true,
            "access_privileges": [
                "user",
                "admin"
            ]
        });
        assert_eq!(parsed, expected);
        println!("{}", parsed);
        Ok(())
    }
}

mod toml {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize, Debug)]
    struct Config {
        package: Package,
        dependencies: HashMap<String, String>,
    }

    #[derive(Deserialize, Debug)]
    struct Package {
        name: String,
        version: String,
        authors: Vec<String>,
    }

    pub fn parse_toml() -> Result<(), toml::de::Error> {
        let s = r#"
          [package]
          name = "example_package"
          version = "0.1.0"
          authors = ["You! <you@example.org>"]

          [dependencies]
          serde = "1.0"
          "#;
        let parsed: toml::Value = toml::from_str(s)?;
        assert_eq!(parsed["dependencies"]["serde"].as_str(), Some("1.0"));
        assert_eq!(parsed["package"]["name"].as_str(), Some("example_package"));
        println!("{:?}", parsed);

        let parsed: Config = toml::from_str(s)?;
        assert_eq!(parsed.dependencies["serde"], "1.0");
        assert_eq!(parsed.package.version, "0.1.0");
        println!("{:?}", parsed);
        Ok(())
    }
}

mod payload {
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
    use std::io::Cursor;

    #[derive(PartialEq, Debug)]
    struct Payload {
        kind: u8,
        value: u16,
    }

    pub fn payload() -> Result<(), std::io::Error> {
        let original = Payload {
            kind: 42,
            value: 123,
        };

        let mut encoded = vec![];
        encoded.write_u8(original.kind)?;
        encoded.write_u16::<LittleEndian>(original.value)?;

        let mut cursor = Cursor::new(encoded);
        let decoded = Payload {
            kind: cursor.read_u8()?,
            value: cursor.read_u16::<LittleEndian>()?,
        };
        assert_eq!(original, decoded);
        println!("{:?}", decoded);
        Ok(())
    }
}

fn main() {
    json::parse_json().unwrap();
    toml::parse_toml().unwrap();
    payload::payload().unwrap();
}
