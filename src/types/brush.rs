//! Brush type for tiling patterns.
//!
//! Brushes define repeating pixel patterns using positional colour tokens.
//! Unlike stamps (which use semantic tokens like `$`, `.`, `x`), brushes use
//! letters (`A`, `B`, `C`) that are bound to colours at usage time.
//!
//! This allows the same brush pattern to be used with different colour schemes.

use std::collections::HashMap;

use crate::types::Colour;

/// A brush definition - a tiling pattern with positional colour tokens.
#[derive(Debug, Clone)]
pub struct Brush {
    /// Brush name (unique identifier).
    pub name: String,

    /// Pattern grid using positional tokens (A, B, C, etc.).
    /// Row-major: pattern[y][x].
    pattern: Vec<Vec<char>>,
}

impl Brush {
    /// Create a new brush.
    pub fn new(name: impl Into<String>, pattern: Vec<Vec<char>>) -> Self {
        Self {
            name: name.into(),
            pattern,
        }
    }

    /// Create a 1x1 brush with a single token.
    pub fn single(name: impl Into<String>, token: char) -> Self {
        Self::new(name, vec![vec![token]])
    }

    /// Get the width of the pattern in pixels.
    pub fn width(&self) -> usize {
        self.pattern.first().map_or(0, |row| row.len())
    }

    /// Get the height of the pattern in pixels.
    pub fn height(&self) -> usize {
        self.pattern.len()
    }

    /// Get the dimensions as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    /// Check if the brush is empty.
    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty() || self.width() == 0
    }

    /// Get a token at the given position (with wrapping for tiling).
    pub fn get(&self, x: usize, y: usize) -> Option<char> {
        if self.is_empty() {
            return None;
        }
        let x = x % self.width();
        let y = y % self.height();
        self.pattern.get(y).and_then(|row| row.get(x)).copied()
    }

    /// Get a reference to the pattern grid.
    pub fn pattern(&self) -> &[Vec<char>] {
        &self.pattern
    }

    /// Get all unique tokens used in this brush.
    pub fn tokens(&self) -> Vec<char> {
        let mut tokens: Vec<char> = self
            .pattern
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .collect();
        tokens.sort();
        tokens.dedup();
        tokens
    }

    /// Render the brush pattern to colours using the given bindings.
    ///
    /// Tokens not in the bindings map will be transparent.
    pub fn render(&self, bindings: &HashMap<char, Colour>) -> Vec<Vec<Colour>> {
        self.pattern
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&token| {
                        bindings
                            .get(&token)
                            .copied()
                            .unwrap_or(Colour::TRANSPARENT)
                    })
                    .collect()
            })
            .collect()
    }

    /// Render a single pixel at (x, y) with tiling.
    pub fn render_pixel(&self, x: usize, y: usize, bindings: &HashMap<char, Colour>) -> Colour {
        self.get(x, y)
            .and_then(|token| bindings.get(&token).copied())
            .unwrap_or(Colour::TRANSPARENT)
    }

    /// Fill a region with this brush pattern.
    ///
    /// Returns a width x height grid of colours.
    pub fn fill(&self, width: usize, height: usize, bindings: &HashMap<char, Colour>) -> Vec<Vec<Colour>> {
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| self.render_pixel(x, y, bindings))
                    .collect()
            })
            .collect()
    }
}

/// Collection of builtin brushes.
pub struct BuiltinBrushes;

impl BuiltinBrushes {
    /// Get all builtin brushes.
    pub fn all() -> Vec<Brush> {
        vec![
            // solid: single A token
            Brush::single("solid", 'A'),
            // checker: 2x2 checkerboard
            Brush::new(
                "checker",
                vec![vec!['A', 'B'], vec!['B', 'A']],
            ),
            // diagonal-r: diagonal lines going right (/)
            // Same base pattern as checker, but application differs
            Brush::new(
                "diagonal-r",
                vec![vec!['A', 'B'], vec!['B', 'A']],
            ),
            // diagonal-l: diagonal lines going left (\)
            Brush::new(
                "diagonal-l",
                vec![vec!['B', 'A'], vec!['A', 'B']],
            ),
            // h-line: horizontal stripes
            Brush::new("h-line", vec![vec!['A'], vec!['B']]),
            // v-line: vertical stripes
            Brush::new("v-line", vec![vec!['A', 'B']]),
            // noise: 4x4 pseudo-random pattern
            Brush::new(
                "noise",
                vec![
                    vec!['A', 'B', 'B', 'A'],
                    vec!['B', 'A', 'A', 'B'],
                    vec!['A', 'A', 'B', 'B'],
                    vec!['B', 'B', 'A', 'A'],
                ],
            ),
        ]
    }

    /// Get a builtin brush by name.
    pub fn get(name: &str) -> Option<Brush> {
        Self::all().into_iter().find(|b| b.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_single() {
        let brush = Brush::single("test", 'A');
        assert_eq!(brush.name, "test");
        assert_eq!(brush.width(), 1);
        assert_eq!(brush.height(), 1);
        assert_eq!(brush.get(0, 0), Some('A'));
    }

    #[test]
    fn test_brush_checker() {
        let brush = Brush::new("checker", vec![vec!['A', 'B'], vec!['B', 'A']]);

        assert_eq!(brush.width(), 2);
        assert_eq!(brush.height(), 2);
        assert_eq!(brush.get(0, 0), Some('A'));
        assert_eq!(brush.get(1, 0), Some('B'));
        assert_eq!(brush.get(0, 1), Some('B'));
        assert_eq!(brush.get(1, 1), Some('A'));
    }

    #[test]
    fn test_brush_tiling() {
        let brush = Brush::new("checker", vec![vec!['A', 'B'], vec!['B', 'A']]);

        // Tiling should wrap around
        // Pattern is:
        //   (0,0)=A  (1,0)=B
        //   (0,1)=B  (1,1)=A
        assert_eq!(brush.get(2, 0), Some('A')); // (2%2, 0%2) = (0, 0) = A
        assert_eq!(brush.get(3, 0), Some('B')); // (3%2, 0%2) = (1, 0) = B
        assert_eq!(brush.get(3, 1), Some('A')); // (3%2, 1%2) = (1, 1) = A
        assert_eq!(brush.get(4, 4), Some('A')); // (4%2, 4%2) = (0, 0) = A
    }

    #[test]
    fn test_brush_tokens() {
        let brush = Brush::new("test", vec![vec!['A', 'B', 'C'], vec!['B', 'A', 'C']]);
        let tokens = brush.tokens();
        assert_eq!(tokens, vec!['A', 'B', 'C']);
    }

    #[test]
    fn test_brush_render() {
        let brush = Brush::new("checker", vec![vec!['A', 'B'], vec!['B', 'A']]);

        let mut bindings = HashMap::new();
        bindings.insert('A', Colour::BLACK);
        bindings.insert('B', Colour::WHITE);

        let rendered = brush.render(&bindings);
        assert_eq!(rendered[0][0], Colour::BLACK);
        assert_eq!(rendered[0][1], Colour::WHITE);
        assert_eq!(rendered[1][0], Colour::WHITE);
        assert_eq!(rendered[1][1], Colour::BLACK);
    }

    #[test]
    fn test_brush_render_missing_binding() {
        let brush = Brush::new("test", vec![vec!['A', 'B']]);

        let mut bindings = HashMap::new();
        bindings.insert('A', Colour::BLACK);
        // B is not bound

        let rendered = brush.render(&bindings);
        assert_eq!(rendered[0][0], Colour::BLACK);
        assert_eq!(rendered[0][1], Colour::TRANSPARENT); // Unbound -> transparent
    }

    #[test]
    fn test_brush_fill() {
        let brush = Brush::new("checker", vec![vec!['A', 'B'], vec!['B', 'A']]);

        let mut bindings = HashMap::new();
        bindings.insert('A', Colour::BLACK);
        bindings.insert('B', Colour::WHITE);

        let filled = brush.fill(4, 4, &bindings);

        // Should tile the 2x2 pattern
        assert_eq!(filled.len(), 4);
        assert_eq!(filled[0].len(), 4);

        // Check pattern repeats
        assert_eq!(filled[0][0], Colour::BLACK);
        assert_eq!(filled[0][2], Colour::BLACK);
        assert_eq!(filled[2][0], Colour::BLACK);
        assert_eq!(filled[2][2], Colour::BLACK);
    }

    #[test]
    fn test_builtin_brushes() {
        let brushes = BuiltinBrushes::all();
        assert_eq!(brushes.len(), 7);

        // Check solid
        let solid = BuiltinBrushes::get("solid").unwrap();
        assert_eq!(solid.size(), (1, 1));
        assert_eq!(solid.get(0, 0), Some('A'));

        // Check checker
        let checker = BuiltinBrushes::get("checker").unwrap();
        assert_eq!(checker.size(), (2, 2));

        // Check h-line
        let hline = BuiltinBrushes::get("h-line").unwrap();
        assert_eq!(hline.size(), (1, 2));
        assert_eq!(hline.get(0, 0), Some('A'));
        assert_eq!(hline.get(0, 1), Some('B'));

        // Check v-line
        let vline = BuiltinBrushes::get("v-line").unwrap();
        assert_eq!(vline.size(), (2, 1));
        assert_eq!(vline.get(0, 0), Some('A'));
        assert_eq!(vline.get(1, 0), Some('B'));

        // Check noise
        let noise = BuiltinBrushes::get("noise").unwrap();
        assert_eq!(noise.size(), (4, 4));
    }

    #[test]
    fn test_builtin_get_unknown() {
        assert!(BuiltinBrushes::get("nonexistent").is_none());
    }
}
