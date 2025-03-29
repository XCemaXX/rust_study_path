use std::{io::Cursor, ops::Range};

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
        ColorType::Rgb | ColorType::Rgba => {
            let chunk_size = if info.color_type == ColorType::Rgb {
                3
            } else {
                4
            };
            process_pixels(chunk_size, |chunk| {
                Color::new(
                    chunk[0] as f32 / 255.,
                    chunk[1] as f32 / 255.,
                    chunk[2] as f32 / 255.,
                )
            })
        }
        ColorType::Grayscale | ColorType::GrayscaleAlpha => {
            let chunk_size = if info.color_type == ColorType::Grayscale {
                1
            } else {
                2
            };
            process_pixels(chunk_size, |chunk| {
                let v = chunk[0] as f32 / 255.;
                Color::new(v, v, v)
            })
        }
        _ => return Err("Unsupported color type".into()),
    };
    if height != pixels.len() {
        return Err("Fail to get pixel info from png".into());
    }
    Ok(pixels)
}

pub fn clamp(range: &Range<f32>, x: f32) -> f32 {
    x.clamp(range.start, range.end)
}
