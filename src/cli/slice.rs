//! Slice command implementation.
//!
//! Imports a PNG spritesheet and generates px definition files from it.

use std::path::PathBuf;

use clap::Args;

use crate::error::{PxError, Result};
use crate::output::{display_path, plural, Printer};

/// A single cell extracted from a spritesheet grid.
pub struct SlicedCell {
    pub name: String,
    pub image: image::RgbaImage,
    pub row: u32,
    pub col: u32,
}

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

/// Returns true if every pixel in the image has alpha == 0.
fn is_fully_transparent(img: &image::RgbaImage) -> bool {
    img.pixels().all(|p| p[3] == 0)
}

/// Split an image into uniform grid cells.
///
/// Calculates grid dimensions from image size and cell size, extracts each
/// sub-image, skips fully transparent cells, and warns about partial edges.
fn slice_grid(
    img: &image::RgbaImage,
    cell_w: u32,
    cell_h: u32,
    base_name: &str,
    printer: &Printer,
) -> Vec<SlicedCell> {
    let mut cols = img.width() / cell_w;
    let mut rows = img.height() / cell_h;

    let partial_x = img.width() % cell_w != 0;
    let partial_y = img.height() % cell_h != 0;

    if partial_x {
        printer.warning(
            "Warning",
            &format!(
                "Image width {} is not divisible by cell width {}; partial column included",
                img.width(),
                cell_w
            ),
        );
        cols += 1;
    }
    if partial_y {
        printer.warning(
            "Warning",
            &format!(
                "Image height {} is not divisible by cell height {}; partial row included",
                img.height(),
                cell_h
            ),
        );
        rows += 1;
    }

    let mut cells = Vec::new();
    let mut skipped = 0u32;

    for row in 0..rows {
        for col in 0..cols {
            let x = col * cell_w;
            let y = row * cell_h;

            // Clamp dimensions for partial edge cells
            let w = cell_w.min(img.width() - x);
            let h = cell_h.min(img.height() - y);

            let sub = image::imageops::crop_imm(img, x, y, w, h).to_image();

            if is_fully_transparent(&sub) {
                skipped += 1;
                continue;
            }

            cells.push(SlicedCell {
                name: format!("{}-{}-{}", base_name, row, col),
                image: sub,
                row,
                col,
            });
        }
    }

    printer.status(
        "Slicing",
        &format!("{}x{} grid ({}x{} cells)", cols, rows, cell_w, cell_h),
    );

    if skipped > 0 {
        printer.info(
            "Finished",
            &format!(
                "{} ({} empty, skipped)",
                plural(cells.len(), "cell", "cells"),
                skipped
            ),
        );
    } else {
        printer.info(
            "Finished",
            &plural(cells.len(), "cell", "cells"),
        );
    }

    cells
}

pub fn run(args: SliceArgs, printer: &Printer) -> Result<Vec<SlicedCell>> {
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

    // Parse --stamp-size if provided
    if let Some(ref stamp_str) = args.stamp_size {
        let (sw, sh) = parse_dimensions(stamp_str)?;
        printer.verbose("Stamp size", &format!("{}x{}", sw, sh));
    }

    // Resolve output directory (default: current directory)
    let _output = args.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Resolve asset name (default: input file stem)
    let base_name = args.name.clone().unwrap_or_else(|| {
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

    // Slice the image
    let _cells = if let Some(ref cell_str) = args.cell {
        let (cw, ch) = parse_dimensions(cell_str)?;
        printer.verbose("Cell size", &format!("{}x{}", cw, ch));
        slice_grid(&img, cw, ch, &base_name, printer)
    } else {
        // No --cell: treat entire image as a single cell
        printer.info("Finished", &plural(1, "cell", "cells"));
        vec![SlicedCell {
            name: base_name,
            image: img,
            row: 0,
            col: 0,
        }]
    };

    Ok(_cells)
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

    // -- is_fully_transparent --

    #[test]
    fn test_is_fully_transparent() {
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 0]));
        assert!(is_fully_transparent(&img));
    }

    #[test]
    fn test_is_not_fully_transparent() {
        let mut img = image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 0]));
        img.put_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        assert!(!is_fully_transparent(&img));
    }

    // -- slice_grid --

    fn test_printer() -> Printer {
        Printer::new()
    }

    #[test]
    fn test_slice_grid_uniform() {
        // 4x4 image with 2x2 cells → 2x2 grid = 4 cells
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 0, 0, 255]));
        let cells = slice_grid(&img, 2, 2, "test", &test_printer());

        assert_eq!(cells.len(), 4);
        assert_eq!((cells[0].row, cells[0].col), (0, 0));
        assert_eq!((cells[1].row, cells[1].col), (0, 1));
        assert_eq!((cells[2].row, cells[2].col), (1, 0));
        assert_eq!((cells[3].row, cells[3].col), (1, 1));

        // Each cell should be 2x2
        for cell in &cells {
            assert_eq!((cell.image.width(), cell.image.height()), (2, 2));
        }
    }

    #[test]
    fn test_slice_grid_skips_transparent() {
        // 4x2 image: left 2x2 opaque, right 2x2 transparent
        let mut img = image::RgbaImage::from_pixel(4, 2, image::Rgba([0, 0, 0, 0]));
        for y in 0..2 {
            for x in 0..2 {
                img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            }
        }

        let cells = slice_grid(&img, 2, 2, "test", &test_printer());
        assert_eq!(cells.len(), 1);
        assert_eq!((cells[0].row, cells[0].col), (0, 0));
    }

    #[test]
    fn test_slice_grid_partial_edge() {
        // 5x3 image with 2x2 cells → 3x2 grid (partial column and row)
        let img = image::RgbaImage::from_pixel(5, 3, image::Rgba([255, 0, 0, 255]));
        let cells = slice_grid(&img, 2, 2, "test", &test_printer());

        assert_eq!(cells.len(), 6); // 3 cols x 2 rows

        // Check partial edge cell dimensions
        let last_col_cell = cells.iter().find(|c| c.col == 2 && c.row == 0).unwrap();
        assert_eq!(last_col_cell.image.width(), 1); // 5 - 4 = 1
        assert_eq!(last_col_cell.image.height(), 2);

        let last_row_cell = cells.iter().find(|c| c.row == 1 && c.col == 0).unwrap();
        assert_eq!(last_row_cell.image.width(), 2);
        assert_eq!(last_row_cell.image.height(), 1); // 3 - 2 = 1
    }

    #[test]
    fn test_slice_grid_single_cell() {
        // Image same size as cell → 1 cell
        let img = image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 0, 0, 255]));
        let cells = slice_grid(&img, 8, 8, "sprite", &test_printer());

        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].name, "sprite-0-0");
        assert_eq!((cells[0].row, cells[0].col), (0, 0));
        assert_eq!((cells[0].image.width(), cells[0].image.height()), (8, 8));
    }

    #[test]
    fn test_slice_grid_names() {
        let img = image::RgbaImage::from_pixel(6, 4, image::Rgba([255, 0, 0, 255]));
        let cells = slice_grid(&img, 2, 2, "sheet", &test_printer());

        assert_eq!(cells.len(), 6);
        assert_eq!(cells[0].name, "sheet-0-0");
        assert_eq!(cells[1].name, "sheet-0-1");
        assert_eq!(cells[2].name, "sheet-0-2");
        assert_eq!(cells[3].name, "sheet-1-0");
        assert_eq!(cells[4].name, "sheet-1-1");
        assert_eq!(cells[5].name, "sheet-1-2");
    }
}
