#[macro_use]
extern crate anyhow;

use std::env;
use std::fs;

use anyhow::Result;
use image::imageops;
use image::load_from_memory;
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use lib_annotate_image::{get_orientation, get_timestamp, Orientation};
use rusttype::{Font, Scale};

fn main() -> Result<()> {
    let (source, target) = if env::args().count() == 3 {
        (env::args().nth(1).unwrap(), env::args().nth(2).unwrap())
    } else {
        return Err(anyhow!("Please enter a source and target file path"));
    };
    process_image(source, target)?;
    Ok(())
}

fn process_image(source: String, target: String) -> Result<()> {
    let file_buffer = fs::read(source)?;
    let timestamp = get_timestamp(&file_buffer)?;
    let orientation = get_orientation(&file_buffer)?;
    let image = load_from_memory(&file_buffer)?;

    let mut image = match orientation {
        Orientation::RotatedLeft => imageops::rotate90(&image),
        Orientation::RotatedRight => imageops::rotate270(&image),
        Orientation::UpsideDown => imageops::rotate180(&image),
        _ => image.to_rgba(),
    };

    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = Font::try_from_vec(font).ok_or(anyhow!("font doesn't exist"))?;

    let (width, height) = image.dimensions();
    let max = width.max(height);

    let scale = Scale {
        x: max as f32 / 20.0,
        y: max as f32 / 20.0,
    };
    draw_text_mut(
        &mut image,
        Rgba([0u8, 0u8, 0u8, 0u8]),
        max / 100,
        max / 100,
        scale,
        &font,
        &timestamp,
    );

    image.save(&target)?;
    Ok(())
}
