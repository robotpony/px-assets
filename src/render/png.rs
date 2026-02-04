//! PNG output for rendered shapes.
//!
//! Converts rendered shapes to PNG files with optional integer scaling.

use std::path::Path;

use image::{ImageBuffer, Rgba, RgbaImage};

use crate::error::{PxError, Result};
use crate::types::Colour;

use super::RenderedShape;

/// Write a rendered shape to a PNG file.
///
/// # Arguments
///
/// * `rendered` - The rendered shape to write
/// * `path` - Output file path
/// * `scale` - Integer scale factor (1 = no scaling)
pub fn write_png(rendered: &RenderedShape, path: &Path, scale: u32) -> Result<()> {
    let scale = scale.max(1); // Minimum scale of 1

    let width = rendered.width() as u32 * scale;
    let height = rendered.height() as u32 * scale;

    let mut img: RgbaImage = ImageBuffer::new(width, height);

    for (y, row) in rendered.pixels().iter().enumerate() {
        for (x, colour) in row.iter().enumerate() {
            let rgba = Rgba(colour.to_rgba());

            // Fill scaled pixels
            for sy in 0..scale {
                for sx in 0..scale {
                    let px = x as u32 * scale + sx;
                    let py = y as u32 * scale + sy;
                    img.put_pixel(px, py, rgba);
                }
            }
        }
    }

    img.save(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: format!("Failed to write PNG: {}", e),
    })?;

    Ok(())
}

/// Scale a rendered shape's pixels by an integer factor.
///
/// Uses nearest-neighbour scaling for crisp pixel art.
pub fn scale_pixels(pixels: &[Vec<Colour>], scale: u32) -> Vec<Vec<Colour>> {
    if scale <= 1 {
        return pixels.to_vec();
    }

    let height = pixels.len();
    let width = pixels.first().map_or(0, |r| r.len());

    let new_height = height * scale as usize;
    let new_width = width * scale as usize;

    let mut scaled = vec![vec![Colour::TRANSPARENT; new_width]; new_height];

    for (y, row) in pixels.iter().enumerate() {
        for (x, &colour) in row.iter().enumerate() {
            for sy in 0..scale as usize {
                for sx in 0..scale as usize {
                    let nx = x * scale as usize + sx;
                    let ny = y * scale as usize + sy;
                    scaled[ny][nx] = colour;
                }
            }
        }
    }

    scaled
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_png_simple() {
        let pixels = vec![
            vec![Colour::BLACK, Colour::WHITE],
            vec![Colour::WHITE, Colour::BLACK],
        ];
        let rendered = RenderedShape::new("test", pixels);

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.png");

        write_png(&rendered, &path, 1).unwrap();

        assert!(path.exists());

        // Read back and verify
        let img = image::open(&path).unwrap().to_rgba8();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.get_pixel(0, 0).0, [0, 0, 0, 255]); // Black
        assert_eq!(img.get_pixel(1, 0).0, [255, 255, 255, 255]); // White
    }

    #[test]
    fn test_write_png_scaled() {
        let pixels = vec![vec![Colour::rgb(255, 0, 0), Colour::rgb(0, 255, 0)]];
        let rendered = RenderedShape::new("test", pixels);

        let dir = tempdir().unwrap();
        let path = dir.path().join("scaled.png");

        write_png(&rendered, &path, 2).unwrap();

        let img = image::open(&path).unwrap().to_rgba8();
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 2);

        // Check that scaling filled correctly
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]); // Red
        assert_eq!(img.get_pixel(1, 0).0, [255, 0, 0, 255]); // Red (scaled)
        assert_eq!(img.get_pixel(2, 0).0, [0, 255, 0, 255]); // Green
        assert_eq!(img.get_pixel(3, 0).0, [0, 255, 0, 255]); // Green (scaled)
    }

    #[test]
    fn test_write_png_with_transparency() {
        let pixels = vec![vec![Colour::TRANSPARENT, Colour::new(255, 0, 0, 128)]];
        let rendered = RenderedShape::new("test", pixels);

        let dir = tempdir().unwrap();
        let path = dir.path().join("alpha.png");

        write_png(&rendered, &path, 1).unwrap();

        let img = image::open(&path).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [0, 0, 0, 0]); // Transparent
        assert_eq!(img.get_pixel(1, 0).0, [255, 0, 0, 128]); // Semi-transparent red
    }

    #[test]
    fn test_scale_pixels() {
        let pixels = vec![vec![Colour::BLACK, Colour::WHITE]];

        let scaled = scale_pixels(&pixels, 2);

        assert_eq!(scaled.len(), 2);
        assert_eq!(scaled[0].len(), 4);

        // First pixel scaled
        assert_eq!(scaled[0][0], Colour::BLACK);
        assert_eq!(scaled[0][1], Colour::BLACK);
        assert_eq!(scaled[1][0], Colour::BLACK);
        assert_eq!(scaled[1][1], Colour::BLACK);

        // Second pixel scaled
        assert_eq!(scaled[0][2], Colour::WHITE);
        assert_eq!(scaled[0][3], Colour::WHITE);
    }

    #[test]
    fn test_scale_pixels_no_scale() {
        let pixels = vec![vec![Colour::BLACK]];
        let scaled = scale_pixels(&pixels, 1);
        assert_eq!(scaled, pixels);
    }

    #[test]
    fn test_write_png_scale_zero_treated_as_one() {
        let pixels = vec![vec![Colour::BLACK]];
        let rendered = RenderedShape::new("test", pixels);

        let dir = tempdir().unwrap();
        let path = dir.path().join("zero.png");

        write_png(&rendered, &path, 0).unwrap();

        let img = image::open(&path).unwrap().to_rgba8();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }
}
