//! Slice command implementation.
//!
//! Imports a PNG spritesheet and generates px definition files from it.

use std::path::PathBuf;

use clap::Args;

use crate::error::{PxError, Result};
use crate::output::{display_path, plural, Printer};
use crate::types::Colour;

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

/// Returns `Some(rgba)` if every pixel in row `y` shares the same RGBA value.
fn uniform_row_colour(img: &image::RgbaImage, y: u32) -> Option<[u8; 4]> {
    let first = img.get_pixel(0, y).0;
    for x in 1..img.width() {
        if img.get_pixel(x, y).0 != first {
            return None;
        }
    }
    Some(first)
}

/// Returns `Some(rgba)` if every pixel in column `x` shares the same RGBA value.
fn uniform_col_colour(img: &image::RgbaImage, x: u32) -> Option<[u8; 4]> {
    let first = img.get_pixel(x, 0).0;
    for y in 1..img.height() {
        if img.get_pixel(x, y).0 != first {
            return None;
        }
    }
    Some(first)
}

/// Groups consecutive separator indices into bands, then returns the content
/// ranges (gaps between bands) as `Vec<(start, width)>`.
fn collapse_separators(indices: &[u32], total_extent: u32) -> Vec<(u32, u32)> {
    if indices.is_empty() {
        return Vec::new();
    }

    // Group consecutive indices into bands: Vec<(band_start, band_end_exclusive)>
    let mut bands: Vec<(u32, u32)> = Vec::new();
    let mut band_start = indices[0];
    let mut band_end = indices[0] + 1;

    for &idx in &indices[1..] {
        if idx == band_end {
            band_end = idx + 1;
        } else {
            bands.push((band_start, band_end));
            band_start = idx;
            band_end = idx + 1;
        }
    }
    bands.push((band_start, band_end));

    // Extract content ranges from gaps between bands
    let mut ranges = Vec::new();

    // Content before first band
    if bands[0].0 > 0 {
        ranges.push((0, bands[0].0));
    }

    // Content between consecutive bands
    for w in bands.windows(2) {
        let gap_start = w[0].1;
        let gap_end = w[1].0;
        if gap_end > gap_start {
            ranges.push((gap_start, gap_end - gap_start));
        }
    }

    // Content after last band
    let last_end = bands.last().unwrap().1;
    if last_end < total_extent {
        ranges.push((last_end, total_extent - last_end));
    }

    ranges
}

/// Returns the most frequent `u32` in a slice. Ties broken by smaller value.
fn most_common_value(values: &[u32]) -> Option<u32> {
    if values.is_empty() {
        return None;
    }

    let mut counts = std::collections::HashMap::new();
    for &v in values {
        *counts.entry(v).or_insert(0u32) += 1;
    }

    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        .map(|(v, _)| v)
}

/// A grid detected by scanning for separator rows and columns.
struct DetectedGrid {
    col_ranges: Vec<(u32, u32)>, // (start_x, width) per column
    row_ranges: Vec<(u32, u32)>, // (start_y, height) per row
    cell_w: u32,                  // most common column width
    cell_h: u32,                  // most common row height
}

/// Attempt to auto-detect a grid by scanning for uniform separator rows/columns.
fn detect_grid(
    img: &image::RgbaImage,
    separator_hex: Option<&str>,
    printer: &Printer,
) -> Result<Option<DetectedGrid>> {
    // Parse explicit separator colour if provided
    let explicit_colour = match separator_hex {
        Some(hex) => {
            let c = Colour::from_hex(hex)?;
            Some([c.r, c.g, c.b, c.a])
        }
        None => None,
    };

    // Scan all rows for uniform colour
    let mut uniform_rows: Vec<(u32, [u8; 4])> = Vec::new();
    for y in 0..img.height() {
        if let Some(rgba) = uniform_row_colour(img, y) {
            uniform_rows.push((y, rgba));
        }
    }

    // Scan all columns for uniform colour
    let mut uniform_cols: Vec<(u32, [u8; 4])> = Vec::new();
    for x in 0..img.width() {
        if let Some(rgba) = uniform_col_colour(img, x) {
            uniform_cols.push((x, rgba));
        }
    }

    // Determine separator colour
    let sep_colour = if let Some(c) = explicit_colour {
        c
    } else {
        // Auto-detect: count frequency of each uniform colour across rows + cols
        let mut freq: std::collections::HashMap<[u8; 4], u32> = std::collections::HashMap::new();
        for &(_, rgba) in &uniform_rows {
            *freq.entry(rgba).or_insert(0) += 1;
        }
        for &(_, rgba) in &uniform_cols {
            *freq.entry(rgba).or_insert(0) += 1;
        }

        match freq.into_iter().max_by_key(|&(_, count)| count) {
            Some((colour, _)) => colour,
            None => return Ok(None), // No uniform rows or columns at all
        }
    };

    // Filter to only rows/columns matching separator colour
    let sep_row_indices: Vec<u32> = uniform_rows
        .iter()
        .filter(|&&(_, rgba)| rgba == sep_colour)
        .map(|&(y, _)| y)
        .collect();

    let sep_col_indices: Vec<u32> = uniform_cols
        .iter()
        .filter(|&&(_, rgba)| rgba == sep_colour)
        .map(|&(x, _)| x)
        .collect();

    // Collapse into content ranges
    let row_ranges = collapse_separators(&sep_row_indices, img.height());
    let col_ranges = collapse_separators(&sep_col_indices, img.width());

    // Need at least 2 content ranges on at least one axis
    if row_ranges.len() < 2 && col_ranges.len() < 2 {
        return Ok(None);
    }

    // If only one axis has separators, the other gets a single full-extent range
    let row_ranges = if row_ranges.is_empty() {
        vec![(0, img.height())]
    } else {
        row_ranges
    };
    let col_ranges = if col_ranges.is_empty() {
        vec![(0, img.width())]
    } else {
        col_ranges
    };

    // Derive most common cell dimensions
    let col_widths: Vec<u32> = col_ranges.iter().map(|&(_, w)| w).collect();
    let row_heights: Vec<u32> = row_ranges.iter().map(|&(_, h)| h).collect();

    let cell_w = most_common_value(&col_widths).unwrap_or(col_widths[0]);
    let cell_h = most_common_value(&row_heights).unwrap_or(row_heights[0]);

    printer.verbose(
        "Separator",
        &format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            sep_colour[0], sep_colour[1], sep_colour[2], sep_colour[3]
        ),
    );

    Ok(Some(DetectedGrid {
        col_ranges,
        row_ranges,
        cell_w,
        cell_h,
    }))
}

/// Extract cells from an auto-detected grid at their actual separator-bounded positions.
fn slice_detected_grid(
    img: &image::RgbaImage,
    grid: &DetectedGrid,
    base_name: &str,
    printer: &Printer,
) -> Vec<SlicedCell> {
    let rows = grid.row_ranges.len();
    let cols = grid.col_ranges.len();

    printer.status(
        "Slicing",
        &format!("{}x{} grid ({}x{} cells)", cols, rows, grid.cell_w, grid.cell_h),
    );

    let mut cells = Vec::new();
    let mut skipped = 0u32;

    for (row_idx, &(y, h)) in grid.row_ranges.iter().enumerate() {
        for (col_idx, &(x, w)) in grid.col_ranges.iter().enumerate() {
            let sub = image::imageops::crop_imm(img, x, y, w, h).to_image();

            if is_fully_transparent(&sub) {
                skipped += 1;
                continue;
            }

            cells.push(SlicedCell {
                name: format!("{}-{}-{}", base_name, row_idx, col_idx),
                image: sub,
                row: row_idx as u32,
                col: col_idx as u32,
            });
        }
    }

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
        // No --cell: attempt auto-detection
        if let Some(grid) = detect_grid(&img, args.separator.as_deref(), printer)? {
            printer.info(
                "Detected",
                &format!(
                    "{}x{} cells ({}x{} grid)",
                    grid.cell_w, grid.cell_h,
                    grid.col_ranges.len(), grid.row_ranges.len()
                ),
            );
            slice_detected_grid(&img, &grid, &base_name, printer)
        } else {
            // Fallback: no grid found
            printer.info("Finished", &plural(1, "cell", "cells"));
            vec![SlicedCell {
                name: base_name,
                image: img,
                row: 0,
                col: 0,
            }]
        }
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

    // -- uniform_row_colour --

    #[test]
    fn test_uniform_row_colour_all_same() {
        let img = image::RgbaImage::from_pixel(4, 2, image::Rgba([255, 0, 0, 255]));
        assert_eq!(uniform_row_colour(&img, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_uniform_row_colour_mixed() {
        let mut img = image::RgbaImage::from_pixel(4, 2, image::Rgba([255, 0, 0, 255]));
        img.put_pixel(2, 0, image::Rgba([0, 255, 0, 255]));
        assert_eq!(uniform_row_colour(&img, 0), None);
    }

    #[test]
    fn test_uniform_col_colour_all_same() {
        let img = image::RgbaImage::from_pixel(2, 4, image::Rgba([0, 0, 255, 255]));
        assert_eq!(uniform_col_colour(&img, 0), Some([0, 0, 255, 255]));
    }

    // -- collapse_separators --

    #[test]
    fn test_collapse_single_pixel_separators() {
        // Separators at 0, 8, 16 with extent 24
        // Bands: [0,0], [8,8], [16,16]
        // Content: (1,7), (9,7), (17,7)
        let indices = vec![0, 8, 16];
        let ranges = collapse_separators(&indices, 24);
        assert_eq!(ranges, vec![(1, 7), (9, 7), (17, 7)]);
    }

    #[test]
    fn test_collapse_multi_pixel_separators() {
        // 2px separators at 0,1 and 8,9 with extent 10
        // Bands: [0,1], [8,9]
        // Content: (2, 6)
        let indices = vec![0, 1, 8, 9];
        let ranges = collapse_separators(&indices, 10);
        assert_eq!(ranges, vec![(2, 6)]);
    }

    #[test]
    fn test_collapse_no_leading_separator() {
        // Separator at 4 with extent 10
        // Content before: (0, 4), after: (5, 5)
        let indices = vec![4];
        let ranges = collapse_separators(&indices, 10);
        assert_eq!(ranges, vec![(0, 4), (5, 5)]);
    }

    #[test]
    fn test_collapse_empty() {
        let ranges = collapse_separators(&[], 10);
        assert!(ranges.is_empty());
    }

    // -- most_common_value --

    #[test]
    fn test_most_common_value_majority() {
        assert_eq!(most_common_value(&[8, 8, 16]), Some(8));
    }

    #[test]
    fn test_most_common_value_empty() {
        assert_eq!(most_common_value(&[]), None);
    }

    #[test]
    fn test_most_common_value_tie() {
        // Tie: 8 and 16 each appear once → smaller wins
        assert_eq!(most_common_value(&[8, 16]), Some(8));
    }

    // -- detect_grid --

    /// Build a test image with transparent separator rows and columns.
    /// Layout: 2x2 grid of 4x4 red cells separated by 1px transparent lines.
    /// Total: 9x9 (4 + 1 + 4 wide, 4 + 1 + 4 tall)
    fn make_grid_image_transparent_seps() -> image::RgbaImage {
        let mut img = image::RgbaImage::from_pixel(9, 9, image::Rgba([0, 0, 0, 0]));
        let red = image::Rgba([255, 0, 0, 255]);

        // Fill 4x4 cell at (0,0)
        for y in 0..4 {
            for x in 0..4 {
                img.put_pixel(x, y, red);
            }
        }
        // Fill 4x4 cell at (5,0)
        for y in 0..4 {
            for x in 5..9 {
                img.put_pixel(x, y, red);
            }
        }
        // Fill 4x4 cell at (0,5)
        for y in 5..9 {
            for x in 0..4 {
                img.put_pixel(x, y, red);
            }
        }
        // Fill 4x4 cell at (5,5)
        for y in 5..9 {
            for x in 5..9 {
                img.put_pixel(x, y, red);
            }
        }
        img
    }

    #[test]
    fn test_detect_grid_transparent_separators() {
        let img = make_grid_image_transparent_seps();
        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();

        assert_eq!(grid.col_ranges.len(), 2);
        assert_eq!(grid.row_ranges.len(), 2);
        assert_eq!(grid.cell_w, 4);
        assert_eq!(grid.cell_h, 4);
        assert_eq!(grid.col_ranges[0], (0, 4));
        assert_eq!(grid.col_ranges[1], (5, 4));
        assert_eq!(grid.row_ranges[0], (0, 4));
        assert_eq!(grid.row_ranges[1], (5, 4));
    }

    #[test]
    fn test_detect_grid_coloured_separators() {
        // 2x2 grid of 4x4 red cells separated by 1px magenta lines
        let magenta = image::Rgba([255, 0, 255, 255]);
        let red = image::Rgba([255, 0, 0, 255]);
        let mut img = image::RgbaImage::from_pixel(9, 9, magenta);

        // Fill cells
        for y in 0..4 { for x in 0..4 { img.put_pixel(x, y, red); } }
        for y in 0..4 { for x in 5..9 { img.put_pixel(x, y, red); } }
        for y in 5..9 { for x in 0..4 { img.put_pixel(x, y, red); } }
        for y in 5..9 { for x in 5..9 { img.put_pixel(x, y, red); } }

        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();

        assert_eq!(grid.col_ranges.len(), 2);
        assert_eq!(grid.row_ranges.len(), 2);
        assert_eq!(grid.cell_w, 4);
        assert_eq!(grid.cell_h, 4);
    }

    #[test]
    fn test_detect_grid_explicit_separator() {
        // Same magenta-separated image, but pass --separator
        let magenta = image::Rgba([255, 0, 255, 255]);
        let red = image::Rgba([255, 0, 0, 255]);
        let mut img = image::RgbaImage::from_pixel(9, 9, magenta);

        for y in 0..4 { for x in 0..4 { img.put_pixel(x, y, red); } }
        for y in 0..4 { for x in 5..9 { img.put_pixel(x, y, red); } }
        for y in 5..9 { for x in 0..4 { img.put_pixel(x, y, red); } }
        for y in 5..9 { for x in 5..9 { img.put_pixel(x, y, red); } }

        let p = test_printer();
        let grid = detect_grid(&img, Some("#FF00FF"), &p).unwrap().unwrap();

        assert_eq!(grid.col_ranges.len(), 2);
        assert_eq!(grid.row_ranges.len(), 2);
        assert_eq!(grid.cell_w, 4);
        assert_eq!(grid.cell_h, 4);
    }

    #[test]
    fn test_detect_grid_no_separators() {
        // Uniform opaque image → no grid detected
        let img = image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 0, 0, 255]));
        let p = test_printer();
        let result = detect_grid(&img, None, &p).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_grid_multi_pixel_separator() {
        // 2x2 grid of 4x4 red cells separated by 2px transparent lines
        // Total: 10x10 (4 + 2 + 4 wide, 4 + 2 + 4 tall)
        let mut img = image::RgbaImage::from_pixel(10, 10, image::Rgba([0, 0, 0, 0]));
        let red = image::Rgba([255, 0, 0, 255]);

        for y in 0..4 { for x in 0..4 { img.put_pixel(x, y, red); } }
        for y in 0..4 { for x in 6..10 { img.put_pixel(x, y, red); } }
        for y in 6..10 { for x in 0..4 { img.put_pixel(x, y, red); } }
        for y in 6..10 { for x in 6..10 { img.put_pixel(x, y, red); } }

        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();

        assert_eq!(grid.col_ranges.len(), 2);
        assert_eq!(grid.row_ranges.len(), 2);
        assert_eq!(grid.cell_w, 4);
        assert_eq!(grid.cell_h, 4);
    }

    #[test]
    fn test_detect_grid_single_axis() {
        // Only horizontal separators: row 4 is transparent, content has mixed colours
        // so content rows aren't detected as uniform.
        // 8x9 image: rows 0-3 mixed content, row 4 transparent, rows 5-8 mixed content
        let red = image::Rgba([255, 0, 0, 255]);
        let blue = image::Rgba([0, 0, 255, 255]);
        let mut img = image::RgbaImage::from_pixel(8, 9, red);

        // Make content rows non-uniform by adding a blue pixel in each
        for y in 0..4 {
            img.put_pixel(0, y, blue);
        }
        for y in 5..9 {
            img.put_pixel(0, y, blue);
        }

        // Transparent separator row
        for x in 0..8 {
            img.put_pixel(x, 4, image::Rgba([0, 0, 0, 0]));
        }

        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();

        assert_eq!(grid.row_ranges.len(), 2);
        assert_eq!(grid.row_ranges[0], (0, 4));
        assert_eq!(grid.row_ranges[1], (5, 4));
        // No column separators → single column spanning full width
        assert_eq!(grid.col_ranges.len(), 1);
        assert_eq!(grid.col_ranges[0], (0, 8));
    }

    // -- slice_detected_grid --

    #[test]
    fn test_slice_detected_grid_basic() {
        let img = make_grid_image_transparent_seps();
        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();
        let cells = slice_detected_grid(&img, &grid, "test", &p);

        assert_eq!(cells.len(), 4);
        for cell in &cells {
            assert_eq!(cell.image.width(), 4);
            assert_eq!(cell.image.height(), 4);
        }
    }

    #[test]
    fn test_slice_detected_grid_skips_transparent() {
        // Build image where one cell is fully transparent
        let mut img = make_grid_image_transparent_seps();
        // Clear cell at (5,5)
        for y in 5..9 {
            for x in 5..9 {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            }
        }

        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();
        let cells = slice_detected_grid(&img, &grid, "test", &p);

        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn test_slice_detected_grid_names() {
        let img = make_grid_image_transparent_seps();
        let p = test_printer();
        let grid = detect_grid(&img, None, &p).unwrap().unwrap();
        let cells = slice_detected_grid(&img, &grid, "sheet", &p);

        assert_eq!(cells[0].name, "sheet-0-0");
        assert_eq!(cells[1].name, "sheet-0-1");
        assert_eq!(cells[2].name, "sheet-1-0");
        assert_eq!(cells[3].name, "sheet-1-1");
    }
}
