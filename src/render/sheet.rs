//! Sprite sheet packer.
//!
//! Packs rendered shapes into a single sprite sheet using shelf packing.
//! Outputs a TexturePacker-compatible JSON Hash format for game engine interop.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::error::{PxError, Result};
use crate::types::Colour;

use super::RenderedShape;

/// A frame in the sprite sheet.
#[derive(Debug, Clone)]
pub struct Frame {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Sprite sheet metadata.
pub struct SheetMeta {
    pub frames: Vec<Frame>,
    pub image: String,
    pub size: (u32, u32),
    pub scale: u32,
}

/// Sprite sheet packer using shelf (row-based) packing.
pub struct SheetPacker {
    pub padding: u32,
}

impl SheetPacker {
    pub fn new(padding: u32) -> Self {
        Self { padding }
    }

    /// Pack rendered shapes into a sprite sheet.
    ///
    /// Returns the composited image and frame metadata.
    pub fn pack(&self, sprites: &[RenderedShape]) -> (RenderedShape, SheetMeta) {
        if sprites.is_empty() {
            let empty = RenderedShape::new("sheet", vec![]);
            let meta = SheetMeta {
                frames: vec![],
                image: "sheet.png".to_string(),
                size: (0, 0),
                scale: 1,
            };
            return (empty, meta);
        }

        // Build index sorted by height descending (stable sort preserves name order)
        let mut indices: Vec<usize> = (0..sprites.len()).collect();
        indices.sort_by(|&a, &b| {
            sprites[b]
                .height()
                .cmp(&sprites[a].height())
                .then_with(|| a.cmp(&b))
        });

        // Compute sheet width as smallest power-of-two that fits
        let max_w = sprites.iter().map(|s| s.width() as u32).max().unwrap_or(1);
        let total_area: u32 = sprites
            .iter()
            .map(|s| {
                (s.width() as u32 + self.padding) * (s.height() as u32 + self.padding)
            })
            .sum();
        let sqrt_area = (total_area as f64).sqrt().ceil() as u32;
        let min_width = max_w.max(sqrt_area);
        let sheet_width = next_power_of_two(min_width);

        // Shelf-pack: place sprites left-to-right, new row when full
        let mut frames: Vec<Frame> = Vec::with_capacity(sprites.len());
        let mut cursor_x: u32 = 0;
        let mut cursor_y: u32 = 0;
        let mut row_height: u32 = 0;

        // We'll store placements indexed by original sprite index
        let mut placements: Vec<(u32, u32)> = vec![(0, 0); sprites.len()];

        for &idx in &indices {
            let w = sprites[idx].width() as u32;
            let h = sprites[idx].height() as u32;

            // Does it fit in the current row?
            if cursor_x + w > sheet_width && cursor_x > 0 {
                // Start new row
                cursor_y += row_height + self.padding;
                cursor_x = 0;
                row_height = 0;
            }

            placements[idx] = (cursor_x, cursor_y);
            row_height = row_height.max(h);
            cursor_x += w + self.padding;
        }

        let sheet_height = cursor_y + row_height;

        // Build frames in original sprite order
        for (idx, sprite) in sprites.iter().enumerate() {
            let (x, y) = placements[idx];
            frames.push(Frame {
                name: sprite.name.clone(),
                x,
                y,
                w: sprite.width() as u32,
                h: sprite.height() as u32,
            });
        }

        // Blit sprites onto the canvas
        let mut pixels =
            vec![vec![Colour::TRANSPARENT; sheet_width as usize]; sheet_height as usize];

        for (idx, sprite) in sprites.iter().enumerate() {
            let (ox, oy) = placements[idx];
            for sy in 0..sprite.height() {
                for sx in 0..sprite.width() {
                    if let Some(c) = sprite.get(sx, sy) {
                        pixels[oy as usize + sy][ox as usize + sx] = c;
                    }
                }
            }
        }

        let sheet = RenderedShape::new("sheet", pixels);
        let meta = SheetMeta {
            frames,
            image: "sheet.png".to_string(),
            size: (sheet_width, sheet_height),
            scale: 1,
        };

        (sheet, meta)
    }
}

/// Write sheet metadata as TexturePacker-compatible JSON Hash format.
pub fn write_sheet_json(meta: &SheetMeta, path: &Path) -> Result<()> {
    let output = TexturePackerJson::from_meta(meta);
    let json = serde_json::to_string_pretty(&output).map_err(|e| PxError::Build {
        message: format!("Failed to serialize sheet metadata: {}", e),
        help: None,
    })?;
    fs::write(path, json).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: format!("Failed to write sheet metadata: {}", e),
    })?;
    Ok(())
}

/// Find the smallest power of two >= n.
fn next_power_of_two(n: u32) -> u32 {
    if n == 0 {
        return 1;
    }
    n.next_power_of_two()
}

// --- TexturePacker JSON serialization types ---

#[derive(Serialize)]
struct TexturePackerJson {
    frames: BTreeMap<String, TPFrame>,
    meta: TPMeta,
}

#[derive(Serialize)]
struct TPFrame {
    frame: TPRect,
    rotated: bool,
    trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    sprite_source_size: TPRect,
    #[serde(rename = "sourceSize")]
    source_size: TPSize,
}

#[derive(Serialize)]
struct TPRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct TPSize {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct TPMeta {
    app: String,
    version: String,
    image: String,
    size: TPSize,
    scale: String,
}

impl TexturePackerJson {
    fn from_meta(meta: &SheetMeta) -> Self {
        let s = meta.scale;
        let mut frames = BTreeMap::new();
        for f in &meta.frames {
            frames.insert(
                f.name.clone(),
                TPFrame {
                    frame: TPRect {
                        x: f.x * s,
                        y: f.y * s,
                        w: f.w * s,
                        h: f.h * s,
                    },
                    rotated: false,
                    trimmed: false,
                    sprite_source_size: TPRect {
                        x: 0,
                        y: 0,
                        w: f.w * s,
                        h: f.h * s,
                    },
                    source_size: TPSize { w: f.w * s, h: f.h * s },
                },
            );
        }

        TexturePackerJson {
            frames,
            meta: TPMeta {
                app: "px".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                image: meta.image.clone(),
                size: TPSize {
                    w: meta.size.0 * s,
                    h: meta.size.1 * s,
                },
                scale: meta.scale.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sprite(name: &str, w: usize, h: usize) -> RenderedShape {
        let pixels = vec![vec![Colour::BLACK; w]; h];
        RenderedShape::new(name, pixels)
    }

    #[test]
    fn test_pack_empty() {
        let packer = SheetPacker::new(0);
        let (sheet, meta) = packer.pack(&[]);
        assert_eq!(sheet.width(), 0);
        assert_eq!(sheet.height(), 0);
        assert!(meta.frames.is_empty());
    }

    #[test]
    fn test_pack_single_sprite() {
        let packer = SheetPacker::new(0);
        let sprites = vec![make_sprite("a", 4, 4)];
        let (sheet, meta) = packer.pack(&sprites);

        assert_eq!(meta.frames.len(), 1);
        assert_eq!(meta.frames[0].name, "a");
        assert_eq!(meta.frames[0].x, 0);
        assert_eq!(meta.frames[0].y, 0);
        assert_eq!(meta.frames[0].w, 4);
        assert_eq!(meta.frames[0].h, 4);
        // Sheet should be power-of-two width
        assert!(sheet.width().is_power_of_two() || sheet.width() == 0);
    }

    #[test]
    fn test_pack_multiple_sprites() {
        let packer = SheetPacker::new(0);
        let sprites = vec![
            make_sprite("a", 4, 4),
            make_sprite("b", 4, 4),
            make_sprite("c", 4, 4),
        ];
        let (_sheet, meta) = packer.pack(&sprites);

        assert_eq!(meta.frames.len(), 3);
        // All frames should have valid positions
        for frame in &meta.frames {
            assert_eq!(frame.w, 4);
            assert_eq!(frame.h, 4);
        }
    }

    #[test]
    fn test_pack_with_padding() {
        let packer = SheetPacker::new(2);
        let sprites = vec![
            make_sprite("a", 4, 4),
            make_sprite("b", 4, 4),
        ];
        let (_sheet, meta) = packer.pack(&sprites);

        assert_eq!(meta.frames.len(), 2);
        // With padding, sprites should not overlap
        let a = &meta.frames[0];
        let b = &meta.frames[1];
        let a_right = a.x + a.w;
        let b_right = b.x + b.w;
        let a_bottom = a.y + a.h;
        let b_bottom = b.y + b.h;

        // Either they're on different rows or horizontally separated by padding
        let no_overlap = b.x >= a_right || a.x >= b_right || b.y >= a_bottom || a.y >= b_bottom;
        assert!(no_overlap, "sprites should not overlap: a={:?} b={:?}", (a.x, a.y, a.w, a.h), (b.x, b.y, b.w, b.h));
    }

    #[test]
    fn test_pack_different_sizes() {
        let packer = SheetPacker::new(0);
        let sprites = vec![
            make_sprite("tall", 2, 8),
            make_sprite("wide", 8, 2),
            make_sprite("small", 2, 2),
        ];
        let (_sheet, meta) = packer.pack(&sprites);

        assert_eq!(meta.frames.len(), 3);
        // Verify no frames overlap
        for i in 0..meta.frames.len() {
            for j in (i + 1)..meta.frames.len() {
                let a = &meta.frames[i];
                let b = &meta.frames[j];
                let no_overlap = b.x >= a.x + a.w
                    || a.x >= b.x + b.w
                    || b.y >= a.y + a.h
                    || a.y >= b.y + b.h;
                assert!(
                    no_overlap,
                    "frames {} and {} overlap",
                    a.name, b.name
                );
            }
        }
    }

    #[test]
    fn test_pack_preserves_pixel_data() {
        let packer = SheetPacker::new(0);
        let red = Colour::rgb(255, 0, 0);
        let blue = Colour::rgb(0, 0, 255);

        let sprite_a = RenderedShape::new("a", vec![vec![red]]);
        let sprite_b = RenderedShape::new("b", vec![vec![blue]]);

        let (sheet, meta) = packer.pack(&[sprite_a, sprite_b]);

        // Find where each sprite was placed and verify pixel data
        for frame in &meta.frames {
            let pixel = sheet.get(frame.x as usize, frame.y as usize).unwrap();
            match frame.name.as_str() {
                "a" => assert_eq!(pixel, red),
                "b" => assert_eq!(pixel, blue),
                _ => panic!("unexpected frame"),
            }
        }
    }

    #[test]
    fn test_sheet_width_is_power_of_two() {
        let packer = SheetPacker::new(0);
        // 3 sprites of 5px wide = 15px total width needed, should round to 16
        let sprites = vec![
            make_sprite("a", 5, 1),
            make_sprite("b", 5, 1),
            make_sprite("c", 5, 1),
        ];
        let (_sheet, meta) = packer.pack(&sprites);
        assert!(
            meta.size.0.is_power_of_two(),
            "sheet width {} should be power of two",
            meta.size.0
        );
    }

    #[test]
    fn test_write_sheet_json() {
        let meta = SheetMeta {
            frames: vec![
                Frame {
                    name: "wall".to_string(),
                    x: 0,
                    y: 0,
                    w: 4,
                    h: 4,
                },
            ],
            image: "sheet.png".to_string(),
            size: (8, 4),
            scale: 1,
        };

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sheet.json");
        write_sheet_json(&meta, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify TexturePacker structure
        assert!(parsed["frames"]["wall"].is_object());
        assert_eq!(parsed["frames"]["wall"]["frame"]["x"], 0);
        assert_eq!(parsed["frames"]["wall"]["frame"]["w"], 4);
        assert_eq!(parsed["frames"]["wall"]["rotated"], false);
        assert_eq!(parsed["frames"]["wall"]["trimmed"], false);
        assert_eq!(parsed["meta"]["app"], "px");
        assert_eq!(parsed["meta"]["image"], "sheet.png");
        assert_eq!(parsed["meta"]["size"]["w"], 8);
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(16), 16);
        assert_eq!(next_power_of_two(17), 32);
    }
}
