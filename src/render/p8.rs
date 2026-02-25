//! PICO-8 cartridge output.
//!
//! Quantizes sprites to the PICO-8 16-colour palette, applies optional
//! dithering, and writes a `.p8` cartridge file with the `__gfx__` section.

use std::fmt;
use std::fs;
use std::path::Path;

use crate::error::{PxError, Result};
use crate::types::Colour;

use super::sheet::Frame;
use super::RenderedShape;

/// The fixed PICO-8 sprite sheet dimensions.
pub const P8_WIDTH: usize = 128;
pub const P8_HEIGHT: usize = 128;

/// The standard PICO-8 16-colour palette.
pub const PICO8_PALETTE: [Colour; 16] = [
    Colour::rgb(0, 0, 0),       // 0  black
    Colour::rgb(29, 43, 83),    // 1  dark blue
    Colour::rgb(126, 37, 83),   // 2  dark purple
    Colour::rgb(0, 135, 81),    // 3  dark green
    Colour::rgb(171, 82, 54),   // 4  brown
    Colour::rgb(95, 87, 79),    // 5  dark grey
    Colour::rgb(194, 195, 199), // 6  light grey
    Colour::rgb(255, 241, 232), // 7  white
    Colour::rgb(255, 0, 77),    // 8  red
    Colour::rgb(255, 163, 0),   // 9  orange
    Colour::rgb(255, 236, 39),  // 10 yellow
    Colour::rgb(0, 228, 54),    // 11 green
    Colour::rgb(41, 173, 255),  // 12 blue
    Colour::rgb(131, 118, 156), // 13 indigo
    Colour::rgb(255, 119, 168), // 14 pink
    Colour::rgb(255, 204, 170), // 15 peach
];

/// Dithering method for colour quantization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DitherMethod {
    /// No dithering; direct nearest-colour mapping.
    None,
    /// Ordered dithering using a Bayer 4x4 threshold matrix.
    Ordered,
    /// Floyd-Steinberg error diffusion dithering.
    FloydSteinberg,
}

impl DitherMethod {
    /// Parse a dither method from a string.
    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" => DitherMethod::None,
            "ordered" | "bayer" => DitherMethod::Ordered,
            "floyd-steinberg" | "fs" => DitherMethod::FloydSteinberg,
            _ => DitherMethod::Ordered,
        }
    }
}

impl fmt::Display for DitherMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DitherMethod::None => write!(f, "none"),
            DitherMethod::Ordered => write!(f, "ordered"),
            DitherMethod::FloydSteinberg => write!(f, "floyd-steinberg"),
        }
    }
}

/// Configuration for PICO-8 output.
pub struct P8Config {
    /// Dithering method to use.
    pub dither: DitherMethod,
    /// Palette index to use for transparent pixels (default: 0).
    pub transparent_index: u8,
}

impl Default for P8Config {
    fn default() -> Self {
        Self {
            dither: DitherMethod::Ordered,
            transparent_index: 0,
        }
    }
}

/// Bayer 4x4 ordered dithering threshold matrix.
/// Values are in the range [0, 16) and should be normalized to [-0.5, 0.5)
/// by computing (value / 16.0 - 0.5) * spread.
const BAYER_4X4: [[u8; 4]; 4] = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5],
];

/// Find the nearest PICO-8 palette index for a colour.
///
/// Uses weighted RGB distance that accounts for human colour perception.
/// Transparent pixels return the configured transparent index.
pub fn quantize_nearest(colour: &Colour, transparent_index: u8) -> u8 {
    if colour.is_transparent() {
        return transparent_index;
    }

    let mut best_index: u8 = 0;
    let mut best_dist = u32::MAX;

    for (i, pc) in PICO8_PALETTE.iter().enumerate() {
        let dist = colour_distance(colour, pc);
        if dist < best_dist {
            best_dist = dist;
            best_index = i as u8;
        }
    }

    best_index
}

/// Weighted RGB colour distance.
///
/// Uses the low-cost approximation from https://www.compuphase.com/cmetric.htm
/// which weights channels based on the mean red value, giving better
/// perceptual results than plain Euclidean distance.
fn colour_distance(a: &Colour, b: &Colour) -> u32 {
    let rmean = (a.r as i32 + b.r as i32) / 2;
    let dr = a.r as i32 - b.r as i32;
    let dg = a.g as i32 - b.g as i32;
    let db = a.b as i32 - b.b as i32;

    let r_weight = 2 + (rmean >> 8);
    let g_weight = 4;
    let b_weight = 2 + ((255 - rmean) >> 8);

    (r_weight * dr * dr + g_weight * dg * dg + b_weight * db * db) as u32
}

/// Quantize a pixel grid to PICO-8 palette indices.
///
/// Dispatches to the configured dithering method.
pub fn quantize_sheet(pixels: &[Vec<Colour>], config: &P8Config) -> Vec<Vec<u8>> {
    match config.dither {
        DitherMethod::None => quantize_direct(pixels, config.transparent_index),
        DitherMethod::Ordered => dither_ordered(pixels, config.transparent_index),
        DitherMethod::FloydSteinberg => dither_floyd_steinberg(pixels, config.transparent_index),
    }
}

/// Direct quantization without dithering.
fn quantize_direct(pixels: &[Vec<Colour>], transparent_index: u8) -> Vec<Vec<u8>> {
    pixels
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| quantize_nearest(c, transparent_index))
                .collect()
        })
        .collect()
}

/// Ordered dithering using the Bayer 4x4 threshold matrix.
///
/// Adjusts each pixel's RGB channels by the threshold offset before
/// finding the nearest palette colour. The spread controls how much
/// the threshold shifts the colour.
fn dither_ordered(pixels: &[Vec<Colour>], transparent_index: u8) -> Vec<Vec<u8>> {
    let spread = 32.0_f32; // dither strength

    pixels
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, c)| {
                    if c.is_transparent() {
                        return transparent_index;
                    }

                    let threshold = BAYER_4X4[y % 4][x % 4] as f32 / 16.0 - 0.5;
                    let offset = threshold * spread;

                    let r = (c.r as f32 + offset).clamp(0.0, 255.0) as u8;
                    let g = (c.g as f32 + offset).clamp(0.0, 255.0) as u8;
                    let b = (c.b as f32 + offset).clamp(0.0, 255.0) as u8;

                    let adjusted = Colour::rgb(r, g, b);
                    quantize_nearest(&adjusted, transparent_index)
                })
                .collect()
        })
        .collect()
}

/// Floyd-Steinberg error diffusion dithering.
///
/// Processes pixels left-to-right, top-to-bottom. After quantizing each
/// pixel, the error is distributed to neighbouring pixels:
///
/// ```text
///        *   7/16
///  3/16 5/16 1/16
/// ```
fn dither_floyd_steinberg(pixels: &[Vec<Colour>], transparent_index: u8) -> Vec<Vec<u8>> {
    let height = pixels.len();
    if height == 0 {
        return vec![];
    }
    let width = pixels[0].len();

    // Working buffer with f32 channels for error accumulation
    let mut buf: Vec<Vec<[f32; 3]>> = pixels
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| [c.r as f32, c.g as f32, c.b as f32])
                .collect()
        })
        .collect();

    // Track which pixels are transparent
    let transparent: Vec<Vec<bool>> = pixels
        .iter()
        .map(|row| row.iter().map(|c| c.is_transparent()).collect())
        .collect();

    let mut result = vec![vec![0u8; width]; height];

    for y in 0..height {
        for x in 0..width {
            if transparent[y][x] {
                result[y][x] = transparent_index;
                continue;
            }

            let old = buf[y][x];
            let old_colour = Colour::rgb(
                old[0].clamp(0.0, 255.0) as u8,
                old[1].clamp(0.0, 255.0) as u8,
                old[2].clamp(0.0, 255.0) as u8,
            );

            let idx = quantize_nearest(&old_colour, transparent_index);
            result[y][x] = idx;

            let new = &PICO8_PALETTE[idx as usize];
            let err = [
                old[0] - new.r as f32,
                old[1] - new.g as f32,
                old[2] - new.b as f32,
            ];

            // Distribute error to neighbours
            let neighbours: [(i32, i32, f32); 4] = [
                (1, 0, 7.0 / 16.0),
                (-1, 1, 3.0 / 16.0),
                (0, 1, 5.0 / 16.0),
                (1, 1, 1.0 / 16.0),
            ];

            for (dx, dy, weight) in &neighbours {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;

                    if !transparent[ny][nx] {
                        buf[ny][nx][0] += err[0] * weight;
                        buf[ny][nx][1] += err[1] * weight;
                        buf[ny][nx][2] += err[2] * weight;
                    }
                }
            }
        }
    }

    result
}

/// Partition frames into those that fit within the given bounds and those that don't.
///
/// A frame fits if its bounding box (x + w, y + h) is within (max_w, max_h).
pub fn sprites_that_fit(frames: &[Frame], max_w: u32, max_h: u32) -> (Vec<&Frame>, Vec<&Frame>) {
    let mut fitting = Vec::new();
    let mut truncated = Vec::new();

    for frame in frames {
        if frame.x + frame.w <= max_w && frame.y + frame.h <= max_h {
            fitting.push(frame);
        } else {
            truncated.push(frame);
        }
    }

    (fitting, truncated)
}

/// Write a PICO-8 cartridge file from a rendered sprite sheet.
///
/// The sheet is cropped (or padded) to 128x128 pixels, quantized to the
/// PICO-8 palette, and written as a `.p8` cartridge with the `__gfx__` section.
pub fn write_p8(sheet: &RenderedShape, path: &Path, config: &P8Config) -> Result<()> {
    // Crop/pad the sheet to 128x128
    let mut pixels = Vec::with_capacity(P8_HEIGHT);
    for y in 0..P8_HEIGHT {
        let mut row = Vec::with_capacity(P8_WIDTH);
        for x in 0..P8_WIDTH {
            if y < sheet.height() && x < sheet.width() {
                row.push(sheet.get(x, y).unwrap_or(Colour::TRANSPARENT));
            } else {
                row.push(Colour::TRANSPARENT);
            }
        }
        pixels.push(row);
    }

    // Quantize to palette indices
    let indices = quantize_sheet(&pixels, config);

    // Build cartridge content
    let mut output = String::new();
    output.push_str("pico-8 cartridge // http://www.pico-8.com\n");
    output.push_str("version 42\n");
    output.push_str("__gfx__\n");

    for row in &indices {
        for &idx in row {
            output.push(std::char::from_digit(idx as u32, 16).unwrap_or('0'));
        }
        output.push('\n');
    }

    fs::write(path, output).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: format!("Failed to write P8 cartridge: {}", e),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pico8_palette_has_16_colours() {
        assert_eq!(PICO8_PALETTE.len(), 16);
        // All colours should be opaque
        for c in &PICO8_PALETTE {
            assert!(c.is_opaque());
        }
    }

    #[test]
    fn test_quantize_nearest_exact_match() {
        // Black (index 0) should map to 0
        assert_eq!(quantize_nearest(&Colour::rgb(0, 0, 0), 0), 0);
        // Red (index 8) should map to 8
        assert_eq!(quantize_nearest(&Colour::rgb(255, 0, 77), 0), 8);
        // White-ish (index 7) should map to 7
        assert_eq!(quantize_nearest(&Colour::rgb(255, 241, 232), 0), 7);
    }

    #[test]
    fn test_quantize_nearest_transparent() {
        assert_eq!(quantize_nearest(&Colour::TRANSPARENT, 0), 0);
        assert_eq!(quantize_nearest(&Colour::TRANSPARENT, 5), 5);
        assert_eq!(quantize_nearest(&Colour::new(255, 0, 0, 0), 3), 3);
    }

    #[test]
    fn test_quantize_nearest_closest() {
        // Pure red (#FF0000) should map to PICO-8 red (#FF004D, index 8)
        let idx = quantize_nearest(&Colour::rgb(255, 0, 0), 0);
        assert_eq!(idx, 8);

        // Mid grey maps to indigo (13: #83769C) via weighted distance
        let idx = quantize_nearest(&Colour::rgb(128, 128, 128), 0);
        assert_eq!(idx, 13);
    }

    #[test]
    fn test_dither_none_matches_direct() {
        let pixels = vec![
            vec![Colour::rgb(255, 0, 0), Colour::rgb(0, 255, 0)],
            vec![Colour::rgb(0, 0, 255), Colour::TRANSPARENT],
        ];

        let config = P8Config {
            dither: DitherMethod::None,
            transparent_index: 0,
        };

        let result = quantize_sheet(&pixels, &config);
        let direct = quantize_direct(&pixels, 0);

        assert_eq!(result, direct);
    }

    #[test]
    fn test_dither_ordered_produces_valid_indices() {
        let pixels = vec![
            vec![Colour::rgb(100, 100, 100); 8],
            vec![Colour::rgb(200, 50, 30); 8],
            vec![Colour::rgb(50, 200, 100); 8],
            vec![Colour::rgb(30, 30, 200); 8],
        ];

        let result = dither_ordered(&pixels, 0);

        for row in &result {
            for &idx in row {
                assert!(idx < 16, "index {} out of range", idx);
            }
        }
    }

    #[test]
    fn test_dither_floyd_steinberg_produces_valid_indices() {
        let pixels = vec![
            vec![Colour::rgb(100, 100, 100); 8],
            vec![Colour::rgb(200, 50, 30); 8],
            vec![Colour::rgb(50, 200, 100); 8],
            vec![Colour::rgb(30, 30, 200); 8],
        ];

        let result = dither_floyd_steinberg(&pixels, 0);

        for row in &result {
            for &idx in row {
                assert!(idx < 16, "index {} out of range", idx);
            }
        }
    }

    #[test]
    fn test_dither_floyd_steinberg_empty() {
        let pixels: Vec<Vec<Colour>> = vec![];
        let result = dither_floyd_steinberg(&pixels, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_write_p8_format() {
        let pixels = vec![vec![Colour::rgb(255, 0, 77); 4]; 4]; // PICO-8 red
        let sheet = RenderedShape::new("test", pixels);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.p8");

        let config = P8Config {
            dither: DitherMethod::None,
            transparent_index: 0,
        };

        write_p8(&sheet, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Header
        assert_eq!(lines[0], "pico-8 cartridge // http://www.pico-8.com");
        assert_eq!(lines[1], "version 42");
        assert_eq!(lines[2], "__gfx__");

        // 128 lines of gfx data
        assert_eq!(lines.len(), 3 + P8_HEIGHT);

        // Each gfx line is exactly 128 chars
        for i in 3..lines.len() {
            assert_eq!(lines[i].len(), P8_WIDTH, "line {} has wrong length", i);
        }

        // First 4 rows should have red (index 8) in first 4 columns
        for i in 3..7 {
            assert!(lines[i].starts_with("8888"), "line {}: {}", i, lines[i]);
        }

        // The rest of row 0 should be black (index 0, transparent_index)
        assert!(lines[3][4..].chars().all(|c| c == '0'));
    }

    #[test]
    fn test_sheet_crop_to_128() {
        // Create a sheet larger than 128x128
        let pixels = vec![vec![Colour::rgb(255, 0, 77); 200]; 200];
        let sheet = RenderedShape::new("big", pixels);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("crop.p8");

        let config = P8Config::default();
        write_p8(&sheet, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let gfx_lines: Vec<&str> = content.lines().skip(3).collect();
        assert_eq!(gfx_lines.len(), P8_HEIGHT);
        for line in &gfx_lines {
            assert_eq!(line.len(), P8_WIDTH);
        }
    }

    #[test]
    fn test_sheet_pad_to_128() {
        // Create a tiny sheet (2x2)
        let pixels = vec![vec![Colour::rgb(0, 228, 54); 2]; 2]; // green
        let sheet = RenderedShape::new("tiny", pixels);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pad.p8");

        let config = P8Config {
            dither: DitherMethod::None,
            transparent_index: 0,
        };

        write_p8(&sheet, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let gfx_lines: Vec<&str> = content.lines().skip(3).collect();
        assert_eq!(gfx_lines.len(), P8_HEIGHT);

        // First 2 rows should have green (index 11 = 'b') in first 2 cols
        assert!(gfx_lines[0].starts_with("bb"), "got: {}", gfx_lines[0]);
        assert!(gfx_lines[1].starts_with("bb"), "got: {}", gfx_lines[1]);

        // Row 2+ column 0 should be transparent (index 0)
        assert!(gfx_lines[2].starts_with("0"), "got: {}", gfx_lines[2]);
    }

    #[test]
    fn test_sprites_that_fit() {
        let frames = vec![
            Frame { name: "a".into(), x: 0, y: 0, w: 8, h: 8 },
            Frame { name: "b".into(), x: 8, y: 0, w: 8, h: 8 },
            Frame { name: "c".into(), x: 120, y: 120, w: 16, h: 16 }, // exceeds 128x128
        ];

        let (fit, trunc) = sprites_that_fit(&frames, 128, 128);
        assert_eq!(fit.len(), 2);
        assert_eq!(trunc.len(), 1);
        assert_eq!(trunc[0].name, "c");
    }

    #[test]
    fn test_dither_method_display() {
        assert_eq!(format!("{}", DitherMethod::None), "none");
        assert_eq!(format!("{}", DitherMethod::Ordered), "ordered");
        assert_eq!(format!("{}", DitherMethod::FloydSteinberg), "floyd-steinberg");
    }

    #[test]
    fn test_dither_method_parse() {
        assert_eq!(DitherMethod::from_str_lossy("none"), DitherMethod::None);
        assert_eq!(DitherMethod::from_str_lossy("ordered"), DitherMethod::Ordered);
        assert_eq!(DitherMethod::from_str_lossy("bayer"), DitherMethod::Ordered);
        assert_eq!(DitherMethod::from_str_lossy("floyd-steinberg"), DitherMethod::FloydSteinberg);
        assert_eq!(DitherMethod::from_str_lossy("fs"), DitherMethod::FloydSteinberg);
        // Unknown defaults to ordered
        assert_eq!(DitherMethod::from_str_lossy("unknown"), DitherMethod::Ordered);
    }

    #[test]
    fn test_colour_distance_identical() {
        let c = Colour::rgb(100, 150, 200);
        assert_eq!(colour_distance(&c, &c), 0);
    }

    #[test]
    fn test_colour_distance_black_white() {
        let dist = colour_distance(&Colour::BLACK, &Colour::WHITE);
        assert!(dist > 0);
    }
}
