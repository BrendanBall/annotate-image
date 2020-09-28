use rexif::{ExifError, ExifTag, TagValue};
use thiserror::Error;

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
             return Ok(Orientation::Undefined)
            }
            Err(AnnotateImageError::Unknown(e.to_string()))
        }
    }
}
