use std::{io::ErrorKind, str::FromStr};

use unicode_segmentation::UnicodeSegmentation;

fn graphemes() {
    let name = "José Guimarães\r\n";
    let graphemes = UnicodeSegmentation::graphemes(name, true).collect::<Vec<&str>>();
    assert_eq!(graphemes[3], "é");
    println!("{graphemes:?}");
}

#[derive(Debug, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl FromStr for Color {
    type Err = std::io::Error;

    fn from_str(hex_color: &str) -> Result<Self, Self::Err> {
        let trimmed = hex_color.trim_matches('#');
        if trimmed.len() != 6 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "hex must be 6 digits",
            ));
        }
        let parse_hex = |trimmed| {
            u8::from_str_radix(trimmed, 16)
                .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "bad hex"))
        };
        Ok(Color {
            r: parse_hex(&hex_color[1..3])?,
            g: parse_hex(&hex_color[3..5])?,
            b: parse_hex(&hex_color[5..7])?,
        })
    }
}

fn main() {
    graphemes();

    let code = r"#fa7268";
    if let Ok(c) = Color::from_str(code) {
        println!(r"The RGB color code is: R: {} G: {} B: {}", c.r, c.g, c.b);
    }
    assert_eq!(
        Color::from_str(&r"#fa7268").unwrap(),
        Color {
            r: 250,
            g: 114,
            b: 104
        }
    );
}
