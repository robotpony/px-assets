//! Stamp type for pixel art definitions.
//!
//! Stamps are the basic building blocks of sprites. Each stamp defines
//! a small pixel grid using semantic tokens:
//! - `$` = edge colour (from palette)
//! - `.` = fill colour (from palette)
//! - `x` = transparent
//!
//! Stamps can declare a default glyph character used to place them in shapes.

use crate::types::Colour;

/// A pixel token in a stamp grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelToken {
    /// Edge colour (`$` in source)
    Edge,
    /// Fill colour (`.` in source)
    Fill,
    /// Transparent (`x` in source)
    Transparent,
}

impl PixelToken {
    /// Parse a character into a pixel token.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '$' => Some(PixelToken::Edge),
            '.' => Some(PixelToken::Fill),
            'x' | 'X' => Some(PixelToken::Transparent),
            ' ' => Some(PixelToken::Fill), // Space defaults to fill
            _ => None,
        }
    }

    /// Convert to character for display.
    pub fn to_char(self) -> char {
        match self {
            PixelToken::Edge => '$',
            PixelToken::Fill => '.',
            PixelToken::Transparent => 'x',
        }
    }

    /// Resolve this token to a colour using the given edge and fill colours.
    pub fn resolve(self, edge: Colour, fill: Colour) -> Colour {
        match self {
            PixelToken::Edge => edge,
            PixelToken::Fill => fill,
            PixelToken::Transparent => Colour::TRANSPARENT,
        }
    }
}

/// A stamp definition - a small pixel art pattern.
#[derive(Debug, Clone)]
pub struct Stamp {
    /// Stamp name (unique identifier).
    pub name: String,

    /// Default glyph character for use in shapes.
    /// If None, the stamp can only be referenced by name in legends.
    pub glyph: Option<char>,

    /// Pixel grid (row-major: pixels[y][x]).
    pixels: Vec<Vec<PixelToken>>,
}

impl Stamp {
    /// Create a new stamp.
    pub fn new(name: impl Into<String>, glyph: Option<char>, pixels: Vec<Vec<PixelToken>>) -> Self {
        Self {
            name: name.into(),
            glyph,
            pixels,
        }
    }

    /// Create a 1x1 stamp with a single token.
    pub fn single(name: impl Into<String>, glyph: Option<char>, token: PixelToken) -> Self {
        Self::new(name, glyph, vec![vec![token]])
    }

    /// Get the width of the stamp in pixels.
    pub fn width(&self) -> usize {
        self.pixels.first().map_or(0, |row| row.len())
    }

    /// Get the height of the stamp in pixels.
    pub fn height(&self) -> usize {
        self.pixels.len()
    }

    /// Get the dimensions as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    /// Check if the stamp is empty.
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty() || self.width() == 0
    }

    /// Get a pixel token at the given position.
    /// Returns None if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<PixelToken> {
        self.pixels.get(y).and_then(|row| row.get(x)).copied()
    }

    /// Get a reference to the pixel grid.
    pub fn pixels(&self) -> &[Vec<PixelToken>] {
        &self.pixels
    }

    /// Iterate over all pixels with their positions.
    pub fn iter_pixels(&self) -> impl Iterator<Item = (usize, usize, PixelToken)> + '_ {
        self.pixels.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, &token)| (x, y, token))
        })
    }

    /// Render the stamp to colours using the given edge and fill colours.
    pub fn render(&self, edge: Colour, fill: Colour) -> Vec<Vec<Colour>> {
        self.pixels
            .iter()
            .map(|row| row.iter().map(|&token| token.resolve(edge, fill)).collect())
            .collect()
    }
}

/// Collection of builtin stamps.
pub struct BuiltinStamps;

impl BuiltinStamps {
    /// Get all builtin stamps.
    pub fn all() -> Vec<Stamp> {
        vec![
            // corner: + -> edge pixel
            Stamp::single("corner", Some('+'), PixelToken::Edge),
            // edge-h: - -> edge pixel
            Stamp::single("edge-h", Some('-'), PixelToken::Edge),
            // edge-v: | -> edge pixel
            Stamp::single("edge-v", Some('|'), PixelToken::Edge),
            // solid: # -> edge pixel
            Stamp::single("solid", Some('#'), PixelToken::Edge),
            // fill: . -> fill pixel
            Stamp::single("fill", Some('.'), PixelToken::Fill),
            // transparent: x -> transparent pixel
            Stamp::single("transparent", Some('x'), PixelToken::Transparent),
            // space: (space) -> fill pixel
            Stamp::single("space", Some(' '), PixelToken::Fill),
        ]
    }

    /// Get a builtin stamp by name.
    pub fn get(name: &str) -> Option<Stamp> {
        Self::all().into_iter().find(|s| s.name == name)
    }

    /// Get a builtin stamp by glyph.
    pub fn get_by_glyph(glyph: char) -> Option<Stamp> {
        Self::all().into_iter().find(|s| s.glyph == Some(glyph))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_token_from_char() {
        assert_eq!(PixelToken::from_char('$'), Some(PixelToken::Edge));
        assert_eq!(PixelToken::from_char('.'), Some(PixelToken::Fill));
        assert_eq!(PixelToken::from_char('x'), Some(PixelToken::Transparent));
        assert_eq!(PixelToken::from_char('X'), Some(PixelToken::Transparent));
        assert_eq!(PixelToken::from_char(' '), Some(PixelToken::Fill));
        assert_eq!(PixelToken::from_char('?'), None);
    }

    #[test]
    fn test_pixel_token_resolve() {
        let edge = Colour::rgb(255, 0, 0);
        let fill = Colour::rgb(0, 255, 0);

        assert_eq!(PixelToken::Edge.resolve(edge, fill), edge);
        assert_eq!(PixelToken::Fill.resolve(edge, fill), fill);
        assert_eq!(
            PixelToken::Transparent.resolve(edge, fill),
            Colour::TRANSPARENT
        );
    }

    #[test]
    fn test_stamp_single() {
        let stamp = Stamp::single("test", Some('#'), PixelToken::Edge);
        assert_eq!(stamp.name, "test");
        assert_eq!(stamp.glyph, Some('#'));
        assert_eq!(stamp.width(), 1);
        assert_eq!(stamp.height(), 1);
        assert_eq!(stamp.get(0, 0), Some(PixelToken::Edge));
    }

    #[test]
    fn test_stamp_multi_pixel() {
        let pixels = vec![
            vec![PixelToken::Edge, PixelToken::Edge],
            vec![PixelToken::Fill, PixelToken::Fill],
        ];
        let stamp = Stamp::new("brick", Some('B'), pixels);

        assert_eq!(stamp.width(), 2);
        assert_eq!(stamp.height(), 2);
        assert_eq!(stamp.size(), (2, 2));
        assert_eq!(stamp.get(0, 0), Some(PixelToken::Edge));
        assert_eq!(stamp.get(1, 1), Some(PixelToken::Fill));
        assert_eq!(stamp.get(5, 5), None); // Out of bounds
    }

    #[test]
    fn test_stamp_render() {
        let pixels = vec![
            vec![PixelToken::Edge, PixelToken::Fill],
            vec![PixelToken::Transparent, PixelToken::Edge],
        ];
        let stamp = Stamp::new("test", None, pixels);

        let edge = Colour::rgb(255, 0, 0);
        let fill = Colour::rgb(0, 255, 0);
        let rendered = stamp.render(edge, fill);

        assert_eq!(rendered[0][0], edge);
        assert_eq!(rendered[0][1], fill);
        assert_eq!(rendered[1][0], Colour::TRANSPARENT);
        assert_eq!(rendered[1][1], edge);
    }

    #[test]
    fn test_stamp_iter_pixels() {
        let pixels = vec![
            vec![PixelToken::Edge, PixelToken::Fill],
            vec![PixelToken::Transparent, PixelToken::Edge],
        ];
        let stamp = Stamp::new("test", None, pixels);

        let collected: Vec<_> = stamp.iter_pixels().collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], (0, 0, PixelToken::Edge));
        assert_eq!(collected[1], (1, 0, PixelToken::Fill));
        assert_eq!(collected[2], (0, 1, PixelToken::Transparent));
        assert_eq!(collected[3], (1, 1, PixelToken::Edge));
    }

    #[test]
    fn test_builtin_stamps() {
        let stamps = BuiltinStamps::all();
        assert_eq!(stamps.len(), 7);

        // Check corner
        let corner = BuiltinStamps::get("corner").unwrap();
        assert_eq!(corner.glyph, Some('+'));
        assert_eq!(corner.get(0, 0), Some(PixelToken::Edge));

        // Check fill
        let fill = BuiltinStamps::get("fill").unwrap();
        assert_eq!(fill.glyph, Some('.'));
        assert_eq!(fill.get(0, 0), Some(PixelToken::Fill));

        // Check transparent
        let transparent = BuiltinStamps::get("transparent").unwrap();
        assert_eq!(transparent.glyph, Some('x'));
        assert_eq!(transparent.get(0, 0), Some(PixelToken::Transparent));
    }

    #[test]
    fn test_builtin_get_by_glyph() {
        let stamp = BuiltinStamps::get_by_glyph('+').unwrap();
        assert_eq!(stamp.name, "corner");

        let stamp = BuiltinStamps::get_by_glyph('#').unwrap();
        assert_eq!(stamp.name, "solid");

        assert!(BuiltinStamps::get_by_glyph('?').is_none());
    }
}
