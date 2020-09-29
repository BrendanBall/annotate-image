use rexif::{ExifError, ExifTag, TagValue};
use thiserror::Error;

use image::{imageops, load_from_memory, DynamicImage, ImageError, ImageOutputFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::io;
use std::io::prelude::*;

#[derive(Error, Debug)]
pub enum AnnotateImageError {
    #[error("the attribute for `{0}` was not found")]
    AttributeNotFound(String),
    #[error("unknown annotate image error")]
    Unknown(String),
}

pub fn get_timestamp(buffer: &[u8]) -> Result<String, AnnotateImageError> {
    match rexif::parse_buffer(&buffer) {
        Ok(exif) => {
            for entry in &exif.entries {
                if entry.tag == ExifTag::DateTime {
                    return Ok(entry.value_more_readable.clone());
                }
            }
            Err(AnnotateImageError::AttributeNotFound(
                ExifTag::DateTime.to_string(),
            ))
        }
        Err(e) => {
            if let ExifError::JpegWithoutExif(_) = e {
                return Err(AnnotateImageError::AttributeNotFound(
                    ExifTag::DateTime.to_string(),
                ));
            }
            Err(AnnotateImageError::Unknown(e.to_string()))
        }
    }
}

#[derive(Debug)]
pub enum Orientation {
    Straight,
    UpsideDown,
    RotatedLeft,
    RotatedRight,
    Undefined,
}

pub fn get_orientation(buffer: &[u8]) -> Result<Orientation, AnnotateImageError> {
    match rexif::parse_buffer(&buffer) {
        Ok(exif) => {
            for entry in &exif.entries {
                if entry.tag == ExifTag::Orientation {
                    let orientation = match entry.value {
                        TagValue::U16(ref v) => {
                            let n = v[0];
                            match n {
                                1 => Orientation::Straight,
                                3 => Orientation::UpsideDown,
                                6 => Orientation::RotatedLeft,
                                8 => Orientation::RotatedRight,
                                _ => Orientation::Undefined,
                            }
                        }
                        _ => panic!("Invalid data for this tag"),
                    };
                    return Ok(orientation);
                }
            }
            Ok(Orientation::Undefined)
        }
        Err(e) => {
            if let ExifError::JpegWithoutExif(_) = e {
                return Ok(Orientation::Undefined);
            }
            Err(AnnotateImageError::Unknown(e.to_string()))
        }
    }
}

impl From<ImageError> for AnnotateImageError {
    fn from(err: ImageError) -> Self {
        AnnotateImageError::Unknown(err.to_string())
    }
}

impl From<io::Error> for AnnotateImageError {
    fn from(err: io::Error) -> Self {
        AnnotateImageError::Unknown(err.to_string())
    }
}

pub fn annotate_image<R: Read, W: Write>(
    source: &mut R,
    mut destination: &mut W,
    text: Option<String>,
    font: &Font,
) -> Result<(), AnnotateImageError> {
    let mut source_buffer: Vec<u8> = Vec::new();
    source.read_to_end(&mut source_buffer)?;
    let orientation = get_orientation(&source_buffer)?;
    let image = &load_from_memory(&source_buffer)?;

    let text: String = match text {
        Some(t) => t,
        None => get_timestamp(&source_buffer)?,
    };

    let mut image = match orientation {
        Orientation::RotatedLeft => imageops::rotate90(image),
        Orientation::RotatedRight => imageops::rotate270(image),
        Orientation::UpsideDown => imageops::rotate180(image),
        _ => image.to_rgba(),
    };

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
        font,
        &text,
    );

    let image = DynamicImage::ImageRgba8(image);
    image.write_to(&mut destination, ImageOutputFormat::Jpeg(u8::MAX))?;
    Ok(())
}
