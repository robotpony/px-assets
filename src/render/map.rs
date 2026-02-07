//! Map renderer - composites rendered shapes and generates instance metadata.
//!
//! The renderer places pre-rendered shapes onto a canvas according to the
//! map's ASCII grid and legend, then collects metadata about what was placed
//! where for JSON export.

use std::collections::HashMap;

use crate::error::{PxError, Result};
use crate::types::{Colour, Map, MapInstance, MapMetadata};

use super::RenderedShape;

/// Map renderer that composites shapes and produces metadata.
pub struct MapRenderer {
    /// Available rendered shapes/prefabs keyed by name.
    rendered: HashMap<String, RenderedShape>,
}

impl MapRenderer {
    /// Create a new map renderer.
    pub fn new() -> Self {
        Self {
            rendered: HashMap::new(),
        }
    }

    /// Register a rendered shape (or prefab) for use in compositing.
    pub fn add_rendered(&mut self, shape: RenderedShape) {
        self.rendered.insert(shape.name.clone(), shape);
    }

    /// Render a map, returning both the composited image and instance metadata.
    pub fn render(&self, map: &Map) -> Result<(RenderedShape, MapMetadata)> {
        if map.is_empty() {
            let metadata = MapMetadata {
                name: map.name.clone(),
                size: [1, 1],
                grid: [0, 0],
                cell_size: [1, 1],
                shapes: vec![],
            };
            return Ok((
                RenderedShape::new(&map.name, vec![vec![Colour::TRANSPARENT]]),
                metadata,
            ));
        }

        // Calculate uniform cell size from referenced shapes (excluding "empty")
        let (cell_w, cell_h) = self.cell_size(map)?;

        // Create transparent canvas
        let canvas_w = map.width() * cell_w;
        let canvas_h = map.height() * cell_h;
        let mut pixels = vec![vec![Colour::TRANSPARENT; canvas_w]; canvas_h];

        // Track instances: name -> list of pixel positions
        let mut instance_positions: HashMap<String, Vec<[usize; 2]>> = HashMap::new();

        // Place each referenced shape
        for (cx, cy, glyph) in map.iter_cells() {
            if glyph == ' ' {
                continue;
            }

            let Some(ref_name) = map.get_legend(glyph) else {
                continue;
            };

            // Skip "empty" cells - transparent, no metadata
            if ref_name == "empty" {
                continue;
            }

            let Some(source) = self.rendered.get(ref_name) else {
                return Err(PxError::Build {
                    message: format!(
                        "Map '{}': legend glyph '{}' references '{}' which has not been rendered",
                        map.name, glyph, ref_name
                    ),
                    help: Some("Ensure all referenced shapes are rendered before the map".to_string()),
                });
            };

            // Blit source onto canvas at cell position
            let dest_x = cx * cell_w;
            let dest_y = cy * cell_h;
            blit(&mut pixels, source, dest_x, dest_y);

            // Track instance position
            instance_positions
                .entry(ref_name.to_string())
                .or_default()
                .push([dest_x, dest_y]);
        }

        // Build metadata
        let mut shapes: Vec<MapInstance> = instance_positions
            .into_iter()
            .map(|(name, positions)| MapInstance {
                name,
                tags: vec![],
                positions,
            })
            .collect();

        // Sort for deterministic output
        shapes.sort_by(|a, b| a.name.cmp(&b.name));

        let metadata = MapMetadata {
            name: map.name.clone(),
            size: [canvas_w, canvas_h],
            grid: [map.width(), map.height()],
            cell_size: [cell_w, cell_h],
            shapes,
        };

        Ok((RenderedShape::new(&map.name, pixels), metadata))
    }

    /// Calculate the uniform cell size (max width x max height of all referenced shapes).
    /// Skips "empty" references.
    fn cell_size(&self, map: &Map) -> Result<(usize, usize)> {
        let mut max_w = 1;
        let mut max_h = 1;

        for ref_name in map.referenced_names() {
            if ref_name == "empty" {
                continue;
            }

            let Some(shape) = self.rendered.get(ref_name) else {
                return Err(PxError::Build {
                    message: format!(
                        "Map '{}': references '{}' which has not been rendered",
                        map.name, ref_name
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
    fn test_render_simple_map() {
        let mut renderer = MapRenderer::new();
        renderer.add_rendered(make_rendered("wall", 2, 2, red()));
        renderer.add_rendered(make_rendered("door", 2, 2, blue()));

        let mut legend = HashMap::new();
        legend.insert('W', "wall".to_string());
        legend.insert('D', "door".to_string());

        let map = Map::new(
            "test-map",
            vec![],
            vec![vec!['W', 'W'], vec!['W', 'D']],
            legend,
        );

        let (result, metadata) = renderer.render(&map).unwrap();

        // 2x2 grid, each cell 2x2 = 4x4 pixels
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
        assert_eq!(result.get(0, 0), Some(red()));
        assert_eq!(result.get(2, 0), Some(red()));  // cell (1,0) = wall
        assert_eq!(result.get(0, 2), Some(red()));  // cell (0,1) = wall
        assert_eq!(result.get(2, 2), Some(blue())); // cell (1,1) = door

        // Check metadata
        assert_eq!(metadata.name, "test-map");
        assert_eq!(metadata.size, [4, 4]);
        assert_eq!(metadata.grid, [2, 2]);
        assert_eq!(metadata.cell_size, [2, 2]);
        assert_eq!(metadata.shapes.len(), 2);
    }

    #[test]
    fn test_render_map_with_empty() {
        let mut renderer = MapRenderer::new();
        renderer.add_rendered(make_rendered("wall", 2, 2, red()));

        let mut legend = HashMap::new();
        legend.insert('W', "wall".to_string());
        legend.insert('.', "empty".to_string());

        let map = Map::new(
            "sparse",
            vec![],
            vec![vec!['W', '.'], vec!['.', 'W']],
            legend,
        );

        let (result, metadata) = renderer.render(&map).unwrap();

        // 2x2 grid, each cell 2x2 = 4x4 pixels
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);

        // Top-left: wall (red)
        assert_eq!(result.get(0, 0), Some(red()));
        assert_eq!(result.get(1, 1), Some(red()));

        // Top-right: empty (transparent)
        assert_eq!(result.get(2, 0), Some(Colour::TRANSPARENT));
        assert_eq!(result.get(3, 1), Some(Colour::TRANSPARENT));

        // Bottom-right: wall (red)
        assert_eq!(result.get(2, 2), Some(red()));

        // Metadata should only contain wall, not empty
        assert_eq!(metadata.shapes.len(), 1);
        assert_eq!(metadata.shapes[0].name, "wall");
        assert_eq!(metadata.shapes[0].positions.len(), 2);
    }

    #[test]
    fn test_render_map_metadata_positions() {
        let mut renderer = MapRenderer::new();
        renderer.add_rendered(make_rendered("block", 4, 4, green()));

        let mut legend = HashMap::new();
        legend.insert('B', "block".to_string());

        let map = Map::new(
            "grid",
            vec![],
            vec![vec!['B', 'B'], vec!['B', 'B']],
            legend,
        );

        let (_, metadata) = renderer.render(&map).unwrap();

        assert_eq!(metadata.shapes.len(), 1);

        let block = &metadata.shapes[0];
        assert_eq!(block.name, "block");
        // 4 instances at pixel positions
        let mut positions = block.positions.clone();
        positions.sort();
        assert_eq!(positions, vec![[0, 0], [0, 4], [4, 0], [4, 4]]);
    }

    #[test]
    fn test_render_map_missing_shape_error() {
        let renderer = MapRenderer::new();

        let mut legend = HashMap::new();
        legend.insert('X', "nonexistent".to_string());

        let map = Map::new(
            "bad",
            vec![],
            vec![vec!['X']],
            legend,
        );

        let result = renderer.render(&map);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_empty_map() {
        let renderer = MapRenderer::new();
        let map = Map::new("empty", vec![], vec![], HashMap::new());

        let (result, metadata) = renderer.render(&map).unwrap();
        assert_eq!(result.width(), 1);
        assert_eq!(result.height(), 1);
        assert!(metadata.shapes.is_empty());
    }

    #[test]
    fn test_render_map_space_skips() {
        let mut renderer = MapRenderer::new();
        renderer.add_rendered(make_rendered("block", 2, 2, green()));

        let mut legend = HashMap::new();
        legend.insert('B', "block".to_string());

        let map = Map::new(
            "spaced",
            vec![],
            vec![vec!['B', ' ', 'B']],
            legend,
        );

        let (result, _) = renderer.render(&map).unwrap();

        // 3 cols x 1 row, each cell 2x2 = 6x2 pixels
        assert_eq!(result.width(), 6);
        assert_eq!(result.height(), 2);

        // First cell: green
        assert_eq!(result.get(0, 0), Some(green()));
        // Middle cell: transparent (space)
        assert_eq!(result.get(2, 0), Some(Colour::TRANSPARENT));
        // Last cell: green
        assert_eq!(result.get(4, 0), Some(green()));
    }
}
