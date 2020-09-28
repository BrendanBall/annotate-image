#[macro_use]
extern crate anyhow;

use std::fs;

use std::path::PathBuf;
use structopt::StructOpt;

use anyhow::Result;
use image::imageops;
use image::load_from_memory;
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use lib_annotate_image::{get_orientation, get_timestamp, Orientation};
use rusttype::{Font, Scale};

#[derive(Debug, StructOpt)]
#[structopt(name = "Annotate Image", about = "Annotate an image. By default it annotates the timestamp")]
struct Opt {
    /// Annotate text
    #[structopt(short, long, required_if("timestamp", "false"))]
    text: Option<String>,

    /// Source image file
    #[structopt(parse(from_os_str))]
    source: PathBuf,

    /// Destination image file
    #[structopt(parse(from_os_str))]
    destination: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let file_buffer = fs::read(opt.source)?;
    let text : String = match opt.text {
        Some(t) => t,
        None => get_timestamp(&file_buffer)?
    };
    process_image(&file_buffer, opt.destination, text)?;
    Ok(())
}

fn process_image(file_buffer: &[u8], target: PathBuf, text: String) -> Result<()> {
    let orientation =  get_orientation(&file_buffer)?;
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
        &text,
    );

    image.save(&target)?;
    Ok(())
}
