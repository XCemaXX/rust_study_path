use ansi_term::{Colour, Style};

fn main() {
    use Colour::*;
    println!(
        "{}: {} {} {} {} {} {} {}",
        Style::new().bold().paint("Rainbow"),
        Red.italic().paint("Red"),
        RGB(255, 165, 0).paint("orange"),
        Yellow.paint("yellow"),
        Green.paint("green"),
        RGB(128, 166, 255).paint("light blue"),
        Blue.paint("blue"),
        Purple.paint("purple"),
    );
}
