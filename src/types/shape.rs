//! Shape type for ASCII sprite compositions.
//!
//! Shapes are the primary way to define sprites in px. They use ASCII art
//! grids where each character maps to a stamp or brush via a legend.
//!
//! # Example
//!
//! ```markdown
//! ---
//! name: wall-segment
//! tags: #wall #solid
//! ---
//!
//! ```px
//! +--+
//! |..|
//! |..|
//! +--+
//! ```
//!
//! ---
//! B: brick
//! ```

use std::collections::HashMap;

/// A shape definition - an ASCII grid that maps to stamps/brushes.
#[derive(Debug, Clone)]
pub struct Shape {
    /// Shape name (unique identifier).
    pub name: String,

    /// Tags for metadata (e.g., "wall", "solid").
    pub tags: Vec<String>,

    /// ASCII grid (row-major: grid[y][x]).
    grid: Vec<Vec<char>>,

    /// Legend mappings (glyph -> stamp/brush reference).
    legend: HashMap<char, LegendEntry>,
}

/// A legend entry describing what a glyph maps to.
#[derive(Debug, Clone, PartialEq)]
pub enum LegendEntry {
    /// Reference to a stamp by name: `B: brick`
    StampRef(String),

    /// Reference to a brush with colour bindings: `{ stamp: checker, A: $edge }`
    BrushRef {
        name: String,
        bindings: HashMap<char, String>,
    },

    /// Fill mode brush (tiled): `{ fill: checker, A: $edge }`
    Fill {
        name: String,
        bindings: HashMap<char, String>,
    },
}

impl Shape {
    /// Create a new shape.
    pub fn new(
        name: impl Into<String>,
        tags: Vec<String>,
        grid: Vec<Vec<char>>,
        legend: HashMap<char, LegendEntry>,
    ) -> Self {
        Self {
            name: name.into(),
            tags,
            grid,
            legend,
        }
    }

    /// Get the width of the shape in cells.
    pub fn width(&self) -> usize {
        self.grid.first().map_or(0, |row| row.len())
    }

    /// Get the height of the shape in cells.
    pub fn height(&self) -> usize {
        self.grid.len()
    }

    /// Get the dimensions as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    /// Check if the shape is empty.
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty() || self.width() == 0
    }

    /// Get a character at the given position.
    pub fn get(&self, x: usize, y: usize) -> Option<char> {
        self.grid.get(y).and_then(|row| row.get(x)).copied()
    }

    /// Get a reference to the grid.
    pub fn grid(&self) -> &[Vec<char>] {
        &self.grid
    }

    /// Get the legend entry for a glyph, if defined.
    pub fn get_legend(&self, glyph: char) -> Option<&LegendEntry> {
        self.legend.get(&glyph)
    }

    /// Get a reference to the full legend.
    pub fn legend(&self) -> &HashMap<char, LegendEntry> {
        &self.legend
    }

    /// Check if a glyph has a legend entry.
    pub fn has_legend(&self, glyph: char) -> bool {
        self.legend.contains_key(&glyph)
    }

    /// Iterate over all cells with their positions.
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, char)> + '_ {
        self.grid.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, &c)| (x, y, c))
        })
    }

    /// Get all unique glyphs used in this shape.
    pub fn glyphs(&self) -> Vec<char> {
        let mut glyphs: Vec<char> = self
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .collect();
        glyphs.sort();
        glyphs.dedup();
        glyphs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_new() {
        let grid = vec![
            vec!['+', '-', '+'],
            vec!['|', '.', '|'],
            vec!['+', '-', '+'],
        ];
        let shape = Shape::new("test", vec![], grid, HashMap::new());

        assert_eq!(shape.name, "test");
        assert_eq!(shape.width(), 3);
        assert_eq!(shape.height(), 3);
    }

    #[test]
    fn test_shape_get() {
        let grid = vec![
            vec!['A', 'B'],
            vec!['C', 'D'],
        ];
        let shape = Shape::new("test", vec![], grid, HashMap::new());

        assert_eq!(shape.get(0, 0), Some('A'));
        assert_eq!(shape.get(1, 0), Some('B'));
        assert_eq!(shape.get(0, 1), Some('C'));
        assert_eq!(shape.get(1, 1), Some('D'));
        assert_eq!(shape.get(5, 5), None);
    }

    #[test]
    fn test_shape_with_legend() {
        let grid = vec![vec!['B', 'B']];
        let mut legend = HashMap::new();
        legend.insert('B', LegendEntry::StampRef("brick".to_string()));

        let shape = Shape::new("test", vec![], grid, legend);

        assert!(shape.has_legend('B'));
        assert!(!shape.has_legend('X'));

        if let Some(LegendEntry::StampRef(name)) = shape.get_legend('B') {
            assert_eq!(name, "brick");
        } else {
            panic!("Expected StampRef");
        }
    }

    #[test]
    fn test_shape_with_tags() {
        let shape = Shape::new(
            "test",
            vec!["wall".to_string(), "solid".to_string()],
            vec![vec!['#']],
            HashMap::new(),
        );

        assert_eq!(shape.tags, vec!["wall", "solid"]);
    }

    #[test]
    fn test_shape_iter_cells() {
        let grid = vec![
            vec!['A', 'B'],
            vec!['C', 'D'],
        ];
        let shape = Shape::new("test", vec![], grid, HashMap::new());

        let cells: Vec<_> = shape.iter_cells().collect();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[0], (0, 0, 'A'));
        assert_eq!(cells[1], (1, 0, 'B'));
        assert_eq!(cells[2], (0, 1, 'C'));
        assert_eq!(cells[3], (1, 1, 'D'));
    }

    #[test]
    fn test_shape_glyphs() {
        let grid = vec![
            vec!['A', 'B', 'A'],
            vec!['B', 'C', 'B'],
        ];
        let shape = Shape::new("test", vec![], grid, HashMap::new());

        let glyphs = shape.glyphs();
        assert_eq!(glyphs, vec!['A', 'B', 'C']);
    }

    #[test]
    fn test_legend_entry_variants() {
        let stamp = LegendEntry::StampRef("brick".to_string());
        let brush = LegendEntry::BrushRef {
            name: "checker".to_string(),
            bindings: [('A', "$edge".to_string())].into_iter().collect(),
        };
        let fill = LegendEntry::Fill {
            name: "checker".to_string(),
            bindings: [('A', "$edge".to_string()), ('B', "$fill".to_string())]
                .into_iter()
                .collect(),
        };

        assert!(matches!(stamp, LegendEntry::StampRef(_)));
        assert!(matches!(brush, LegendEntry::BrushRef { .. }));
        assert!(matches!(fill, LegendEntry::Fill { .. }));
    }
}
