use std::io::Cursor;

use crate::Color;
use png::{ColorType, Decoder};

pub fn load_png(png_data: &[u8]) -> Result<Vec<Vec<Color>>, Box<dyn std::error::Error>> {
    let reader = Cursor::new(png_data);
    let decoder = Decoder::new(reader);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;

    let width = info.width as usize;
    let height = info.height as usize;

    let process_pixels = |chunk_size: usize, convert_fn: fn(&[u8]) -> _| {
        buf.chunks_exact(chunk_size * width)
            .map(|row| row.chunks_exact(chunk_size).map(convert_fn).collect())
            .collect()
    };

    let pixels: Vec<_> = match info.color_type {
        ColorType::Rgb => process_pixels(3, |chunk| {
            Color::new(
                chunk[0] as f32 / 255.0,
                chunk[1] as f32 / 255.0,
                chunk[2] as f32 / 255.0,
            )
        }),
        ColorType::Rgba => process_pixels(4, |chunk| {
            Color::new(
                chunk[0] as f32 / 255.0,
                chunk[1] as f32 / 255.0,
                chunk[2] as f32 / 255.0,
            )
        }),
        ColorType::Grayscale => process_pixels(1, |chunk| {
            let v = chunk[0] as f32 / 255.0;
            Color::new(v, v, v)
        }),
        ColorType::GrayscaleAlpha => process_pixels(2, |chunk| {
            let v = chunk[0] as f32 / 255.0;
            Color::new(v, v, v)
        }),
        _ => return Err("Unsupported color type".into()),
    };
    if height != pixels.len() {
        return Err("Fail to get pixel info from png".into());
    }
    Ok(pixels)
}
