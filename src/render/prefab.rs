//! Prefab renderer - composites rendered shapes into larger images.
//!
//! The renderer places pre-rendered shapes onto a canvas according to
//! the prefab's ASCII grid and legend.

use std::collections::HashMap;

use crate::error::{PxError, Result};
use crate::types::{Colour, Prefab};

use super::RenderedShape;

/// Prefab renderer that composites shapes into larger images.
pub struct PrefabRenderer {
    /// Available rendered shapes/prefabs keyed by name.
    rendered: HashMap<String, RenderedShape>,
}

impl PrefabRenderer {
    /// Create a new prefab renderer.
    pub fn new() -> Self {
        Self {
            rendered: HashMap::new(),
        }
    }

    /// Register a rendered shape (or prefab) for use in compositing.
    pub fn add_rendered(&mut self, shape: RenderedShape) {
        self.rendered.insert(shape.name.clone(), shape);
    }

    /// Render a prefab by compositing referenced shapes onto a canvas.
    pub fn render(&self, prefab: &Prefab) -> Result<RenderedShape> {
        if prefab.is_empty() {
            return Ok(RenderedShape::new(
                &prefab.name,
                vec![vec![Colour::TRANSPARENT]],
            ));
        }

        // Calculate uniform cell size from referenced shapes
        let (cell_w, cell_h) = self.cell_size(prefab)?;

        // Create transparent canvas
        let canvas_w = prefab.width() * cell_w;
        let canvas_h = prefab.height() * cell_h;
        let mut pixels = vec![vec![Colour::TRANSPARENT; canvas_w]; canvas_h];

        // Place each referenced shape
        for (cx, cy, glyph) in prefab.iter_cells() {
            if glyph == ' ' {
                continue;
            }

            let Some(ref_name) = prefab.get_legend(glyph) else {
                continue;
            };

            let Some(source) = self.rendered.get(ref_name) else {
                return Err(PxError::Build {
                    message: format!(
                        "Prefab '{}': legend glyph '{}' references '{}' which has not been rendered",
                        prefab.name, glyph, ref_name
                    ),
                    help: Some("Ensure all referenced shapes are rendered before the prefab".to_string()),
                });
            };

            // Blit source onto canvas at cell position
            let dest_x = cx * cell_w;
            let dest_y = cy * cell_h;
            blit(&mut pixels, source, dest_x, dest_y);
        }

        Ok(RenderedShape::new(&prefab.name, pixels))
    }

    /// Calculate the uniform cell size (max width x max height of all referenced shapes).
    fn cell_size(&self, prefab: &Prefab) -> Result<(usize, usize)> {
        let mut max_w = 1;
        let mut max_h = 1;

        for ref_name in prefab.referenced_names() {
            let Some(shape) = self.rendered.get(ref_name) else {
                return Err(PxError::Build {
                    message: format!(
                        "Prefab '{}': references '{}' which has not been rendered",
                        prefab.name, ref_name
                    ),
                    help: None,
                });
            };

            max_w = max_w.max(shape.width());
            max_h = max_h.max(shape.height());
        }

        Ok((max_w, max_h))
    }
}

/// Copy source pixels onto destination at offset, skipping transparent pixels.
fn blit(dest: &mut [Vec<Colour>], source: &RenderedShape, offset_x: usize, offset_y: usize) {
    for sy in 0..source.height() {
        let dy = offset_y + sy;
        if dy >= dest.len() {
            break;
        }
        for sx in 0..source.width() {
            let dx = offset_x + sx;
            if dx >= dest[dy].len() {
                break;
            }
            if let Some(pixel) = source.get(sx, sy) {
                if pixel.a > 0 {
                    dest[dy][dx] = pixel;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Colour {
        Colour::rgb(255, 0, 0)
    }

    fn green() -> Colour {
        Colour::rgb(0, 255, 0)
    }

    fn blue() -> Colour {
        Colour::rgb(0, 0, 255)
    }

    fn make_rendered(name: &str, w: usize, h: usize, colour: Colour) -> RenderedShape {
        RenderedShape::new(name, vec![vec![colour; w]; h])
    }

    #[test]
    fn test_render_simple_prefab() {
        let mut renderer = PrefabRenderer::new();
        renderer.add_rendered(make_rendered("top", 2, 2, red()));
        renderer.add_rendered(make_rendered("bottom", 2, 2, blue()));

        let mut legend = HashMap::new();
        legend.insert('T', "top".to_string());
        legend.insert('B', "bottom".to_string());

        let prefab = Prefab::new(
            "stack",
            vec![],
            vec![vec!['T'], vec!['B']],
            legend,
        );

        let result = renderer.render(&prefab).unwrap();

        // 1 col x 2 rows, each cell 2x2 = 2x4 pixels
        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 4);
        assert_eq!(result.get(0, 0), Some(red()));
        assert_eq!(result.get(0, 2), Some(blue()));
    }

    #[test]
    fn test_render_prefab_with_spaces() {
        let mut renderer = PrefabRenderer::new();
        renderer.add_rendered(make_rendered("block", 2, 2, green()));

        let mut legend = HashMap::new();
        legend.insert('B', "block".to_string());

        let prefab = Prefab::new(
            "spaced",
            vec![],
            vec![vec!['B', ' ', 'B']],
            legend,
        );

        let result = renderer.render(&prefab).unwrap();

        // 3 cols x 1 row, each cell 2x2 = 6x2 pixels
        assert_eq!(result.width(), 6);
        assert_eq!(result.height(), 2);

        // First cell: green
        assert_eq!(result.get(0, 0), Some(green()));
        assert_eq!(result.get(1, 0), Some(green()));

        // Middle cell: transparent (space = empty slot)
        assert_eq!(result.get(2, 0), Some(Colour::TRANSPARENT));
        assert_eq!(result.get(3, 0), Some(Colour::TRANSPARENT));

        // Last cell: green
        assert_eq!(result.get(4, 0), Some(green()));
        assert_eq!(result.get(5, 0), Some(green()));
    }

    #[test]
    fn test_render_prefab_uniform_cell_size() {
        let mut renderer = PrefabRenderer::new();
        // "wide" is 4x1, "tall" is 1x3
        renderer.add_rendered(make_rendered("wide", 4, 1, red()));
        renderer.add_rendered(make_rendered("tall", 1, 3, blue()));

        let mut legend = HashMap::new();
        legend.insert('W', "wide".to_string());
        legend.insert('T', "tall".to_string());

        let prefab = Prefab::new(
            "mixed",
            vec![],
            vec![vec!['W', 'T']],
            legend,
        );

        let result = renderer.render(&prefab).unwrap();

        // Cell size = max(4,1) x max(1,3) = 4x3
        // 2 cols x 1 row = 8x3 pixels
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 3);

        // Wide shape placed at top-left of cell (0,0): fills (0..4, 0)
        assert_eq!(result.get(0, 0), Some(red()));
        assert_eq!(result.get(3, 0), Some(red()));
        // Below wide shape is transparent (it's only 1px tall)
        assert_eq!(result.get(0, 1), Some(Colour::TRANSPARENT));

        // Tall shape placed at top-left of cell (4,0): fills (4, 0..3)
        assert_eq!(result.get(4, 0), Some(blue()));
        assert_eq!(result.get(4, 1), Some(blue()));
        assert_eq!(result.get(4, 2), Some(blue()));
        // Right of tall shape is transparent (it's only 1px wide)
        assert_eq!(result.get(5, 0), Some(Colour::TRANSPARENT));
    }

    #[test]
    fn test_render_nested_prefab() {
        let mut renderer = PrefabRenderer::new();
        renderer.add_rendered(make_rendered("brick", 2, 2, red()));

        // First, render a simple prefab
        let mut legend1 = HashMap::new();
        legend1.insert('B', "brick".to_string());
        let inner = Prefab::new(
            "wall",
            vec![],
            vec![vec!['B', 'B']],
            legend1,
        );

        let rendered_wall = renderer.render(&inner).unwrap();
        // wall = 4x2 (2 cells of 2x2)
        assert_eq!(rendered_wall.width(), 4);
        assert_eq!(rendered_wall.height(), 2);

        // Add rendered wall for the outer prefab
        renderer.add_rendered(rendered_wall);

        // Outer prefab stacks two walls
        let mut legend2 = HashMap::new();
        legend2.insert('W', "wall".to_string());
        let outer = Prefab::new(
            "tower",
            vec![],
            vec![vec!['W'], vec!['W']],
            legend2,
        );

        let result = renderer.render(&outer).unwrap();

        // Cell size = 4x2, grid = 1x2 = 4x4 pixels
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
        // All red
        assert_eq!(result.get(0, 0), Some(red()));
        assert_eq!(result.get(3, 3), Some(red()));
    }

    #[test]
    fn test_render_missing_shape_error() {
        let renderer = PrefabRenderer::new();

        let mut legend = HashMap::new();
        legend.insert('X', "nonexistent".to_string());

        let prefab = Prefab::new(
            "bad",
            vec![],
            vec![vec!['X']],
            legend,
        );

        let result = renderer.render(&prefab);
        assert!(result.is_err());
    }

    #[test]
    fn test_blit_skips_transparent() {
        // Source has a transparent pixel
        let source_pixels = vec![vec![red(), Colour::TRANSPARENT], vec![Colour::TRANSPARENT, blue()]];
        let source = RenderedShape::new("src", source_pixels);

        let mut dest = vec![vec![green(); 2]; 2];
        blit(&mut dest, &source, 0, 0);

        // Red overwrites, transparent leaves green
        assert_eq!(dest[0][0], red());
        assert_eq!(dest[0][1], green());
        assert_eq!(dest[1][0], green());
        assert_eq!(dest[1][1], blue());
    }

    #[test]
    fn test_render_empty_prefab() {
        let renderer = PrefabRenderer::new();
        let prefab = Prefab::new("empty", vec![], vec![], HashMap::new());

        let result = renderer.render(&prefab).unwrap();
        assert_eq!(result.width(), 1);
        assert_eq!(result.height(), 1);
    }
}
