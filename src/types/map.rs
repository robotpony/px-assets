//! Map type for level layouts.
//!
//! Maps are structurally identical to prefabs (ASCII grid + legend referencing
//! shapes/prefabs) but semantically distinct: they represent level layouts
//! rather than reusable components. The key addition is instance metadata
//! generation for JSON output.
//!
//! # Example
//!
//! ```markdown
//! ---
//! name: dungeon-1
//! tags: "#level"
//! ---
//!
//! ```px
//! WWWW
//! W..W
//! W..W
//! WWDW
//! ```
//!
//! ---
//! W: wall-segment
//! D: door
//! .: empty
//! ```

use std::collections::HashMap;

use serde::Serialize;

/// A map definition - an ASCII placement grid representing a level layout.
#[derive(Debug, Clone)]
pub struct Map {
    /// Map name (unique identifier).
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

impl Map {
    /// Create a new map.
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

    /// Create a new map with scale.
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

    /// Get the width of the map in cells.
    pub fn width(&self) -> usize {
        self.grid.first().map_or(0, |row| row.len())
    }

    /// Get the height of the map in cells.
    pub fn height(&self) -> usize {
        self.grid.len()
    }

    /// Get the dimensions as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    /// Check if the map is empty.
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

    /// Get all unique glyphs used in this map.
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

/// Metadata about a rendered map, for JSON export.
#[derive(Debug, Clone, Serialize)]
pub struct MapMetadata {
    /// Map name.
    pub name: String,

    /// Pixel dimensions [width, height].
    pub size: [usize; 2],

    /// Cell dimensions [cols, rows].
    pub grid: [usize; 2],

    /// Pixel size of each cell [width, height].
    pub cell_size: [usize; 2],

    /// Instances placed on the map.
    pub shapes: Vec<MapInstance>,
}

/// A shape/prefab instance placed on the map.
#[derive(Debug, Clone, Serialize)]
pub struct MapInstance {
    /// Shape or prefab name.
    pub name: String,

    /// Tags from the source shape/prefab (if available).
    pub tags: Vec<String>,

    /// Pixel positions where this shape appears.
    pub positions: Vec<[usize; 2]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_new() {
        let grid = vec![
            vec!['W', 'W'],
            vec!['W', 'D'],
        ];
        let mut legend = HashMap::new();
        legend.insert('W', "wall".to_string());
        legend.insert('D', "door".to_string());

        let map = Map::new("dungeon-1", vec![], grid, legend);

        assert_eq!(map.name, "dungeon-1");
        assert_eq!(map.width(), 2);
        assert_eq!(map.height(), 2);
        assert_eq!(map.size(), (2, 2));
    }

    #[test]
    fn test_map_get() {
        let grid = vec![
            vec!['A', 'B'],
            vec!['C', 'D'],
        ];
        let map = Map::new("test", vec![], grid, HashMap::new());

        assert_eq!(map.get(0, 0), Some('A'));
        assert_eq!(map.get(1, 0), Some('B'));
        assert_eq!(map.get(0, 1), Some('C'));
        assert_eq!(map.get(1, 1), Some('D'));
        assert_eq!(map.get(5, 5), None);
    }

    #[test]
    fn test_map_legend_lookup() {
        let grid = vec![vec!['W', 'W']];
        let mut legend = HashMap::new();
        legend.insert('W', "wall-segment".to_string());

        let map = Map::new("test", vec![], grid, legend);

        assert_eq!(map.get_legend('W'), Some("wall-segment"));
        assert_eq!(map.get_legend('X'), None);
        assert!(map.has_legend('W'));
        assert!(!map.has_legend('X'));
    }

    #[test]
    fn test_map_with_tags() {
        let map = Map::new(
            "test",
            vec!["level".to_string(), "dungeon".to_string()],
            vec![vec!['#']],
            HashMap::new(),
        );

        assert_eq!(map.tags, vec!["level", "dungeon"]);
    }

    #[test]
    fn test_map_iter_cells() {
        let grid = vec![
            vec!['A', 'B'],
            vec!['C', 'D'],
        ];
        let map = Map::new("test", vec![], grid, HashMap::new());

        let cells: Vec<_> = map.iter_cells().collect();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[0], (0, 0, 'A'));
        assert_eq!(cells[1], (1, 0, 'B'));
        assert_eq!(cells[2], (0, 1, 'C'));
        assert_eq!(cells[3], (1, 1, 'D'));
    }

    #[test]
    fn test_map_glyphs() {
        let grid = vec![
            vec!['A', 'B', 'A'],
            vec!['B', 'C', 'B'],
        ];
        let map = Map::new("test", vec![], grid, HashMap::new());

        let glyphs = map.glyphs();
        assert_eq!(glyphs, vec!['A', 'B', 'C']);
    }

    #[test]
    fn test_map_referenced_names() {
        let grid = vec![vec!['W', 'D', 'W']];
        let mut legend = HashMap::new();
        legend.insert('W', "wall".to_string());
        legend.insert('D', "door".to_string());

        let map = Map::new("test", vec![], grid, legend);

        let names = map.referenced_names();
        assert_eq!(names, vec!["door", "wall"]);
    }

    #[test]
    fn test_map_with_scale() {
        let grid = vec![vec!['A']];
        let map = Map::with_scale("test", vec![], grid, HashMap::new(), Some(4));

        assert_eq!(map.scale, Some(4));
    }

    #[test]
    fn test_map_empty() {
        let map = Map::new("empty", vec![], vec![], HashMap::new());
        assert!(map.is_empty());
    }

    #[test]
    fn test_map_metadata_serialize() {
        let metadata = MapMetadata {
            name: "test-map".to_string(),
            size: [32, 32],
            grid: [4, 4],
            cell_size: [8, 8],
            shapes: vec![
                MapInstance {
                    name: "wall".to_string(),
                    tags: vec!["solid".to_string()],
                    positions: vec![[0, 0], [8, 0], [16, 0]],
                },
            ],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"name\":\"test-map\""));
        assert!(json.contains("\"size\":[32,32]"));
        assert!(json.contains("\"positions\":[[0,0],[8,0],[16,0]]"));
    }
}
