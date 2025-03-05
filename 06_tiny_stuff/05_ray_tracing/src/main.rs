use std::{env, fs::File, io::BufWriter, process};

use png::{BitDepth, ColorType, Encoder};
use ray_tracing::{Color, render_world};

fn main() {
    let mut args = env::args();
    args.next();
    let output_file = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Specify output png name");
            process::exit(1)
        }
    };

    let data = render_world();

    if let Err(e) = save_as_png(&data.pixels, data.width, data.height, &output_file) {
        eprintln!("Cannot create png file{output_file}:\n  {e}");
        process::exit(1);
    }
}

pub fn save_as_png(
    pixels: &[Color],
    width: u32,
    height: u32,
    filename: &str,
) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, width as u32, height as u32);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let data: Vec<u8> = pixels
        .iter()
        .flat_map(|c| vec![c.r() as u8, c.g() as u8, c.b() as u8])
        .collect();
    writer.write_image_data(&data)?;

    Ok(())
}
