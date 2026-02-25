//! Slice command implementation.
//!
//! Imports a PNG spritesheet and generates px definition files from it.

use std::path::PathBuf;

use clap::Args;

use crate::error::{PxError, Result};
use crate::output::{display_path, Printer};

/// Slice a PNG into sprite definition files
#[derive(Args, Debug)]
pub struct SliceArgs {
    /// PNG file to slice into sprites
    #[arg(required = true)]
    pub input: PathBuf,

    /// Cell size as WxH (e.g. 16x16)
    #[arg(long)]
    pub cell: Option<String>,

    /// Output directory for generated files
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Base name for generated assets (default: input filename stem)
    #[arg(long)]
    pub name: Option<String>,

    /// Enable stamp detection (structural deduplication)
    #[arg(long)]
    pub stamps: bool,

    /// Stamp detection block size as WxH (e.g. 4x4)
    #[arg(long)]
    pub stamp_size: Option<String>,

    /// Separator colour for grid auto-detection (default: auto)
    #[arg(long)]
    pub separator: Option<String>,

    /// Path to existing .palette.md to use instead of generating one
    #[arg(long)]
    pub palette: Option<PathBuf>,
}

/// Parse a "WxH" dimension string into (width, height).
fn parse_dimensions(s: &str) -> Result<(u32, u32)> {
    let parts: Vec<&str> = s.splitn(2, |c| c == 'x' || c == 'X').collect();
    if parts.len() != 2 {
        return Err(PxError::Parse {
            message: format!("Invalid dimensions '{}': expected WxH (e.g. 16x16)", s),
            help: Some("Use the format WxH, for example: 16x16, 8x16".to_string()),
        });
    }

    let w: u32 = parts[0].parse().map_err(|_| PxError::Parse {
        message: format!("Invalid width '{}' in dimensions '{}'", parts[0], s),
        help: Some("Width must be a positive integer".to_string()),
    })?;

    let h: u32 = parts[1].parse().map_err(|_| PxError::Parse {
        message: format!("Invalid height '{}' in dimensions '{}'", parts[1], s),
        help: Some("Height must be a positive integer".to_string()),
    })?;

    if w == 0 || h == 0 {
        return Err(PxError::Parse {
            message: format!("Dimensions must be non-zero, got {}x{}", w, h),
            help: Some("Both width and height must be at least 1".to_string()),
        });
    }

    Ok((w, h))
}

pub fn run(args: SliceArgs, printer: &Printer) -> Result<()> {
    let path = &args.input;
    let display = display_path(path);

    // Validate input path exists
    if !path.exists() {
        return Err(PxError::Io {
            path: path.clone(),
            message: format!("File not found: {}", display),
        });
    }

    // Warn if not a .png file
    if path.extension().and_then(|e| e.to_str()) != Some("png") {
        printer.warning("Warning", &format!("{} does not have a .png extension", display));
    }

    // Load PNG
    printer.status("Loading", &display);

    let img = image::open(path)
        .map_err(|e| PxError::Io {
            path: path.clone(),
            message: format!("Failed to load image: {}", e),
        })?
        .to_rgba8();

    let (w, h) = (img.width(), img.height());

    // Validate non-zero dimensions
    if w == 0 || h == 0 {
        return Err(PxError::Build {
            message: format!("Image has zero dimensions ({}x{})", w, h),
            help: Some("Input image must have non-zero width and height".to_string()),
        });
    }

    // Parse --cell if provided
    if let Some(ref cell_str) = args.cell {
        let (cw, ch) = parse_dimensions(cell_str)?;
        printer.verbose("Cell size", &format!("{}x{}", cw, ch));
    }

    // Parse --stamp-size if provided
    if let Some(ref stamp_str) = args.stamp_size {
        let (sw, sh) = parse_dimensions(stamp_str)?;
        printer.verbose("Stamp size", &format!("{}x{}", sw, sh));
    }

    // Resolve output directory (default: current directory)
    let _output = args.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Resolve asset name (default: input file stem)
    let _name = args.name.clone().unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sprite")
            .to_string()
    });

    let pixels = w as u64 * h as u64;
    printer.info(
        "Analyzed",
        &format!("{}x{} image ({} pixels)", w, h, pixels),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dimensions_valid() {
        assert_eq!(parse_dimensions("16x16").unwrap(), (16, 16));
    }

    #[test]
    fn test_parse_dimensions_rectangular() {
        assert_eq!(parse_dimensions("8x16").unwrap(), (8, 16));
    }

    #[test]
    fn test_parse_dimensions_uppercase() {
        assert_eq!(parse_dimensions("8X16").unwrap(), (8, 16));
    }

    #[test]
    fn test_parse_dimensions_invalid() {
        assert!(parse_dimensions("abc").is_err());
    }

    #[test]
    fn test_parse_dimensions_zero() {
        assert!(parse_dimensions("0x16").is_err());
    }

    #[test]
    fn test_parse_dimensions_zero_height() {
        assert!(parse_dimensions("16x0").is_err());
    }

    #[test]
    fn test_parse_dimensions_non_numeric() {
        assert!(parse_dimensions("axb").is_err());
    }
}
