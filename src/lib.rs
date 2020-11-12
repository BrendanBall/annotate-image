use rexif::{ExifError, ExifTag, TagValue, ExifData};
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
    #[error("the image has no metadata")]
    NoMetadata,
    #[error("unknown annotate image error")]
    Unknown(String),
}

#[derive(Debug)]
pub enum Orientation {
    Straight,
    UpsideDown,
    RotatedLeft,
    RotatedRight,
    Undefined,
}

struct ImageMetadata(ExifData);

impl ImageMetadata {

    pub fn new(buffer: &[u8]) -> Result<Option<Self>, AnnotateImageError> {
        match rexif::parse_buffer(&buffer) {
            Ok(exif) => {
                Ok(Some(Self(exif)))
            },
            Err(e) => {
                if let ExifError::JpegWithoutExif(_) = e {
                return Ok(None)
                }
                Err(AnnotateImageError::Unknown(e.to_string()))
            }
        }
    }

    pub fn timestamp(&self) -> Result<String, AnnotateImageError> {
        for entry in &self.0.entries {
            if entry.tag == ExifTag::DateTime {
                return Ok(entry.value_more_readable.clone());
            }
        }
        Err(AnnotateImageError::AttributeNotFound(
            ExifTag::DateTime.to_string(),
        ))
    } 
    
    pub fn orientation(&self) -> Result<Orientation, AnnotateImageError> {
        for entry in &self.0.entries {
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
    let image_metadata =  ImageMetadata::new(&source_buffer)?;
    let orientation = match &image_metadata {
        Some(m) => m.orientation() ,
        None => Ok(Orientation::Undefined),
    }?;
    let text : String = match text {
        Some(t) => t,
        None => match &image_metadata {
            Some(m) => m.timestamp() ,
            None => Err(AnnotateImageError::AttributeNotFound(ExifTag::DateTime.to_string())),
        }?
    };
    
    let image = &load_from_memory(&source_buffer)?;
   
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
