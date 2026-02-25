use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;

use crate::error::{PxError, Result};
use crate::output::{display_path, plural, Printer};
use crate::types::Colour;

/// Extract a colour palette from a PNG file
#[derive(Args, Debug)]
pub struct PaletteArgs {
    /// PNG file to extract colours from
    #[arg(required = true)]
    pub file: PathBuf,

    /// Maximum number of colours to output
    #[arg(long)]
    pub max: Option<usize>,
}

pub fn run(args: PaletteArgs, printer: &Printer) -> Result<()> {
    let path = &args.file;
    let display = display_path(path);

    let img = image::open(path)
        .map_err(|e| PxError::Io {
            path: path.clone(),
            message: e.to_string(),
        })?
        .to_rgba8();

    // Count pixel frequencies, skipping fully transparent pixels
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for pixel in img.pixels() {
        let rgba = pixel.0;
        if rgba[3] == 0 {
            continue;
        }
        *counts.entry(rgba).or_insert(0) += 1;
    }

    // Sort by frequency (most common first)
    let mut colours: Vec<([u8; 4], usize)> = counts.into_iter().collect();
    colours.sort_by(|a, b| b.1.cmp(&a.1));

    // Apply --max limit
    if let Some(max) = args.max {
        colours.truncate(max);
    }

    let total = colours.len();
    printer.status("Sampled", &format!("{} from {}", plural(total, "colour", "colours"), display));

    // Print palette lines to stdout
    for (i, (rgba, _count)) in colours.iter().enumerate() {
        let colour = Colour::new(rgba[0], rgba[1], rgba[2], rgba[3]);
        println!("$colour-{}: {}", i + 1, colour);
    }

    Ok(())
}
