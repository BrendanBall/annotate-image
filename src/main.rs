#[macro_use]
extern crate anyhow;

use std::path::PathBuf;
use structopt::StructOpt;

use anyhow::Result;
use lib_annotate_image::{annotate_image};
use rusttype::{Font};
use std::fs::File;
use std::io::{BufReader, BufWriter};

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
    
    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = Font::try_from_vec(font).ok_or(anyhow!("font doesn't exist"))?;

    let mut source_file = BufReader::new(File::open(opt.source)?);
    let mut destination_file = BufWriter::new(File::create(opt.destination)?);

    annotate_image(&mut source_file, &mut destination_file, opt.text, &font)?;
    Ok(())
}
