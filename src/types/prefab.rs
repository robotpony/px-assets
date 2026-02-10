//! Prefab type for compositing shapes into larger images.
//!
//! Prefabs use an ASCII placement grid where each character maps to a
//! shape or another prefab via a legend. This allows building complex
//! sprites from smaller, reusable pieces.
//!
//! # Example
//!
//! ```markdown
//! ---
//! name: tower
//! tags: "#building"
//! ---
//!
//! ```px
//! R
//! W
//! W
//! D
//! ```
//!
//! ---
//! R: roof
//! W: wall-segment
//! D: door
//! ```

use std::collections::HashMap;

use serde::Serialize;

/// Metadata about a rendered prefab, for JSON export.
#[derive(Debug, Clone, Serialize)]
pub struct PrefabMetadata {
    /// Prefab name.
    pub name: String,

    /// Pixel dimensions [width, height].
    pub size: [usize; 2],

    /// Tags from frontmatter.
    pub tags: Vec<String>,

    /// Cell dimensions [cols, rows].
    pub grid: [usize; 2],

    /// Pixel size of each cell [width, height].
    pub cell_size: [usize; 2],

    /// Instances placed in the prefab.
    pub shapes: Vec<PrefabInstance>,
}

/// A shape/prefab instance placed in a prefab.
#[derive(Debug, Clone, Serialize)]
pub struct PrefabInstance {
    /// Shape or prefab name.
    pub name: String,

    /// Pixel positions where this shape appears.
    pub positions: Vec<[usize; 2]>,
}

/// A prefab definition - an ASCII placement grid that maps to shapes/prefabs.
#[derive(Debug, Clone)]
pub struct Prefab {
    /// Prefab name (unique identifier).
    pub name: String,

    /// Tags for metadata.
    pub tags: Vec<String>,

    /// ASCII placement grid (row-major: grid[y][x]).
    grid: Vec<Vec<char>>,

    /// Legend mappings (glyph -> shape/prefab name).
    legend: HashMap<char, String>,

    /// Optional scale factor from frontmatter.
    pub scale: Option<u32>,
}

impl Prefab {
    /// Create a new prefab.
    pub fn new(
        name: impl Into<String>,
        tags: Vec<String>,
        grid: Vec<Vec<char>>,
        legend: HashMap<char, String>,
    ) -> Self {
        Self {
            name: name.into(),
            tags,
            grid,
            legend,
            scale: None,
        }
    }

    /// Create a new prefab with scale.
    pub fn with_scale(
        name: impl Into<String>,
        tags: Vec<String>,
        grid: Vec<Vec<char>>,
        legend: HashMap<char, String>,
        scale: Option<u32>,
    ) -> Self {
        Self {
            name: name.into(),
            tags,
            grid,
            legend,
            scale,
        }
    }

    /// Get the width of the prefab in cells.
    pub fn width(&self) -> usize {
        self.grid.first().map_or(0, |row| row.len())
    }

    /// Get the height of the prefab in cells.
    pub fn height(&self) -> usize {
        self.grid.len()
    }

    /// Get the dimensions as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    /// Check if the prefab is empty.
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

    /// Get the referenced name for a glyph.
    pub fn get_legend(&self, glyph: char) -> Option<&str> {
        self.legend.get(&glyph).map(|s| s.as_str())
    }

    /// Get a reference to the full legend.
    pub fn legend(&self) -> &HashMap<char, String> {
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

    /// Get all unique glyphs used in this prefab.
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

    /// Get all unique referenced names from the legend.
    pub fn referenced_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.legend.values().map(|s| s.as_str()).collect();
        names.sort();
        names.dedup();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefab_new() {
        let grid = vec![
            vec!['R'],
            vec!['W'],
            vec!['D'],
        ];
        let mut legend = HashMap::new();
        legend.insert('R', "roof".to_string());
        legend.insert('W', "wall".to_string());
        legend.insert('D', "door".to_string());

        let prefab = Prefab::new("tower", vec![], grid, legend);

        assert_eq!(prefab.name, "tower");
        assert_eq!(prefab.width(), 1);
        assert_eq!(prefab.height(), 3);
        assert_eq!(prefab.size(), (1, 3));
    }

    #[test]
    fn test_prefab_get() {
        let grid = vec![
            vec!['A', 'B'],
            vec!['C', 'D'],
        ];
        let prefab = Prefab::new("test", vec![], grid, HashMap::new());

        assert_eq!(prefab.get(0, 0), Some('A'));
        assert_eq!(prefab.get(1, 0), Some('B'));
        assert_eq!(prefab.get(0, 1), Some('C'));
        assert_eq!(prefab.get(1, 1), Some('D'));
        assert_eq!(prefab.get(5, 5), None);
    }

    #[test]
    fn test_prefab_legend_lookup() {
        let grid = vec![vec!['W', 'W']];
        let mut legend = HashMap::new();
        legend.insert('W', "wall-segment".to_string());

        let prefab = Prefab::new("test", vec![], grid, legend);

        assert_eq!(prefab.get_legend('W'), Some("wall-segment"));
        assert_eq!(prefab.get_legend('X'), None);
        assert!(prefab.has_legend('W'));
        assert!(!prefab.has_legend('X'));
    }

    #[test]
    fn test_prefab_with_tags() {
        let prefab = Prefab::new(
            "test",
            vec!["building".to_string(), "tall".to_string()],
            vec![vec!['#']],
            HashMap::new(),
        );

        assert_eq!(prefab.tags, vec!["building", "tall"]);
    }

    #[test]
    fn test_prefab_iter_cells() {
        let grid = vec![
            vec!['A', 'B'],
            vec!['C', 'D'],
        ];
        let prefab = Prefab::new("test", vec![], grid, HashMap::new());

        let cells: Vec<_> = prefab.iter_cells().collect();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[0], (0, 0, 'A'));
        assert_eq!(cells[1], (1, 0, 'B'));
        assert_eq!(cells[2], (0, 1, 'C'));
        assert_eq!(cells[3], (1, 1, 'D'));
    }

    #[test]
    fn test_prefab_glyphs() {
        let grid = vec![
            vec!['A', 'B', 'A'],
            vec!['B', 'C', 'B'],
        ];
        let prefab = Prefab::new("test", vec![], grid, HashMap::new());

        let glyphs = prefab.glyphs();
        assert_eq!(glyphs, vec!['A', 'B', 'C']);
    }

    #[test]
    fn test_prefab_referenced_names() {
        let grid = vec![vec!['R', 'W', 'W', 'D']];
        let mut legend = HashMap::new();
        legend.insert('R', "roof".to_string());
        legend.insert('W', "wall".to_string());
        legend.insert('D', "door".to_string());

        let prefab = Prefab::new("test", vec![], grid, legend);

        let names = prefab.referenced_names();
        assert_eq!(names, vec!["door", "roof", "wall"]);
    }

    #[test]
    fn test_prefab_with_scale() {
        let grid = vec![vec!['A']];
        let prefab = Prefab::with_scale("test", vec![], grid, HashMap::new(), Some(4));

        assert_eq!(prefab.scale, Some(4));
    }

    #[test]
    fn test_prefab_space_is_empty_slot() {
        let grid = vec![
            vec!['A', ' ', 'B'],
        ];
        let mut legend = HashMap::new();
        legend.insert('A', "left".to_string());
        legend.insert('B', "right".to_string());

        let prefab = Prefab::new("test", vec![], grid, legend);

        assert_eq!(prefab.get(0, 0), Some('A'));
        assert_eq!(prefab.get(1, 0), Some(' '));
        assert_eq!(prefab.get(2, 0), Some('B'));
        // Space has no legend entry
        assert!(!prefab.has_legend(' '));
    }

    #[test]
    fn test_prefab_empty() {
        let prefab = Prefab::new("empty", vec![], vec![], HashMap::new());
        assert!(prefab.is_empty());
    }
}
