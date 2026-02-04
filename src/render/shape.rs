//! Shape renderer - converts shapes to pixel grids.
//!
//! The renderer resolves glyphs to stamps/brushes and applies colours
//! from the palette via a shader.

use std::collections::HashMap;

use crate::types::{
    BuiltinStamps, Brush, Colour, LegendEntry, Palette, PixelToken, Shape, Stamp,
};

/// A rendered shape - a grid of colours.
#[derive(Debug, Clone)]
pub struct RenderedShape {
    /// Shape name.
    pub name: String,

    /// Pixel grid (row-major: pixels[y][x]).
    pixels: Vec<Vec<Colour>>,

    /// Width in pixels.
    width: usize,

    /// Height in pixels.
    height: usize,
}

impl RenderedShape {
    /// Create a new rendered shape.
    pub fn new(name: impl Into<String>, pixels: Vec<Vec<Colour>>) -> Self {
        let height = pixels.len();
        let width = pixels.first().map_or(0, |row| row.len());

        Self {
            name: name.into(),
            pixels,
            width,
            height,
        }
    }

    /// Get the width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the dimensions as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get a pixel at the given position.
    pub fn get(&self, x: usize, y: usize) -> Option<Colour> {
        self.pixels.get(y).and_then(|row| row.get(x)).copied()
    }

    /// Get a reference to the pixel grid.
    pub fn pixels(&self) -> &[Vec<Colour>] {
        &self.pixels
    }

    /// Convert to a flat RGBA buffer (for image output).
    pub fn to_rgba_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(self.width * self.height * 4);
        for row in &self.pixels {
            for colour in row {
                buffer.extend_from_slice(&colour.to_rgba());
            }
        }
        buffer
    }
}

/// Shape renderer configuration and state.
pub struct ShapeRenderer<'a> {
    /// Stamps available for rendering (name -> stamp).
    stamps: HashMap<String, &'a Stamp>,

    /// Brushes available for rendering (name -> brush).
    brushes: HashMap<String, &'a Brush>,

    /// Palette for colour resolution.
    palette: &'a Palette,

    /// Palette variant to use (if any).
    variant: Option<&'a str>,
}

impl<'a> ShapeRenderer<'a> {
    /// Create a new shape renderer.
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            stamps: HashMap::new(),
            brushes: HashMap::new(),
            palette,
            variant: None,
        }
    }

    /// Set the palette variant to use.
    pub fn with_variant(mut self, variant: &'a str) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Add a stamp to the renderer.
    pub fn add_stamp(&mut self, stamp: &'a Stamp) {
        self.stamps.insert(stamp.name.clone(), stamp);
    }

    /// Add multiple stamps to the renderer.
    pub fn add_stamps(&mut self, stamps: impl IntoIterator<Item = &'a Stamp>) {
        for stamp in stamps {
            self.add_stamp(stamp);
        }
    }

    /// Add a brush to the renderer.
    pub fn add_brush(&mut self, brush: &'a Brush) {
        self.brushes.insert(brush.name.clone(), brush);
    }

    /// Add multiple brushes to the renderer.
    pub fn add_brushes(&mut self, brushes: impl IntoIterator<Item = &'a Brush>) {
        for brush in brushes {
            self.add_brush(brush);
        }
    }

    /// Render a shape to pixels.
    pub fn render(&self, shape: &Shape) -> RenderedShape {
        let width = shape.width();
        let height = shape.height();

        let mut pixels = vec![vec![Colour::TRANSPARENT; width]; height];

        for (x, y, glyph) in shape.iter_cells() {
            let colour = self.resolve_glyph(glyph, shape, x, y);
            pixels[y][x] = colour;
        }

        RenderedShape::new(&shape.name, pixels)
    }

    /// Resolve a glyph to a colour.
    fn resolve_glyph(&self, glyph: char, shape: &Shape, x: usize, y: usize) -> Colour {
        // 1. Check shape's legend
        if let Some(entry) = shape.get_legend(glyph) {
            return self.resolve_legend_entry(entry, x, y);
        }

        // 2. Check stamps by glyph
        if let Some(stamp) = self.find_stamp_by_glyph(glyph) {
            return self.render_stamp_pixel(stamp, 0, 0);
        }

        // 3. Check builtin stamps
        if let Some(stamp) = BuiltinStamps::get_by_glyph(glyph) {
            return self.render_stamp_pixel(&stamp, 0, 0);
        }

        // 4. Fallback: magenta for missing glyphs
        Colour::MAGENTA
    }

    /// Resolve a legend entry to a colour.
    fn resolve_legend_entry(&self, entry: &LegendEntry, x: usize, y: usize) -> Colour {
        match entry {
            LegendEntry::StampRef(name) => {
                // Look up stamp by name
                if let Some(stamp) = self.stamps.get(name) {
                    return self.render_stamp_pixel(stamp, 0, 0);
                }
                // Try builtin stamps by name
                if let Some(stamp) = BuiltinStamps::get(name) {
                    return self.render_stamp_pixel(&stamp, 0, 0);
                }
                // Missing stamp
                Colour::MAGENTA
            }

            LegendEntry::BrushRef { name, bindings } => {
                // Look up brush and render single pixel
                if let Some(brush) = self.brushes.get(name) {
                    let colour_bindings = self.resolve_bindings(bindings);
                    return brush.render_pixel(0, 0, &colour_bindings);
                }
                Colour::MAGENTA
            }

            LegendEntry::Fill { name, bindings } => {
                // Look up brush and render with tiling
                if let Some(brush) = self.brushes.get(name) {
                    let colour_bindings = self.resolve_bindings(bindings);
                    return brush.render_pixel(x, y, &colour_bindings);
                }
                Colour::MAGENTA
            }
        }
    }

    /// Find a stamp by its default glyph.
    fn find_stamp_by_glyph(&self, glyph: char) -> Option<&Stamp> {
        self.stamps
            .values()
            .find(|s| s.glyph == Some(glyph))
            .copied()
    }

    /// Render a single pixel from a stamp.
    fn render_stamp_pixel(&self, stamp: &Stamp, x: usize, y: usize) -> Colour {
        let token = stamp.get(x, y).unwrap_or(PixelToken::Transparent);
        self.resolve_token(token)
    }

    /// Resolve a pixel token to a colour using the palette.
    fn resolve_token(&self, token: PixelToken) -> Colour {
        match token {
            PixelToken::Edge => self.get_colour("edge").unwrap_or(Colour::BLACK),
            PixelToken::Fill => self.get_colour("fill").unwrap_or(Colour::WHITE),
            PixelToken::Transparent => Colour::TRANSPARENT,
        }
    }

    /// Resolve brush colour bindings to actual colours.
    fn resolve_bindings(&self, bindings: &HashMap<char, String>) -> HashMap<char, Colour> {
        bindings
            .iter()
            .filter_map(|(&token, colour_ref)| {
                self.resolve_colour_ref(colour_ref).map(|c| (token, c))
            })
            .collect()
    }

    /// Resolve a colour reference (like "$edge" or "#FF0000").
    fn resolve_colour_ref(&self, colour_ref: &str) -> Option<Colour> {
        if colour_ref.starts_with('#') {
            // Hex colour
            Colour::from_hex(colour_ref).ok()
        } else {
            // Palette reference
            let name = colour_ref.strip_prefix('$').unwrap_or(colour_ref);
            self.get_colour(name)
        }
    }

    /// Get a colour from the palette (with variant if set).
    fn get_colour(&self, name: &str) -> Option<Colour> {
        if let Some(variant) = self.variant {
            self.palette.get_with_variant(name, variant)
        } else {
            self.palette.get(name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BuiltinBrushes;

    fn default_palette() -> Palette {
        Palette::default_palette()
    }

    #[test]
    fn test_rendered_shape_new() {
        let pixels = vec![
            vec![Colour::BLACK, Colour::WHITE],
            vec![Colour::WHITE, Colour::BLACK],
        ];
        let rendered = RenderedShape::new("test", pixels);

        assert_eq!(rendered.name, "test");
        assert_eq!(rendered.width(), 2);
        assert_eq!(rendered.height(), 2);
    }

    #[test]
    fn test_rendered_shape_get() {
        let pixels = vec![
            vec![Colour::BLACK, Colour::WHITE],
            vec![Colour::rgb(255, 0, 0), Colour::TRANSPARENT],
        ];
        let rendered = RenderedShape::new("test", pixels);

        assert_eq!(rendered.get(0, 0), Some(Colour::BLACK));
        assert_eq!(rendered.get(1, 0), Some(Colour::WHITE));
        assert_eq!(rendered.get(0, 1), Some(Colour::rgb(255, 0, 0)));
        assert_eq!(rendered.get(1, 1), Some(Colour::TRANSPARENT));
        assert_eq!(rendered.get(5, 5), None);
    }

    #[test]
    fn test_rendered_shape_to_rgba_buffer() {
        let pixels = vec![vec![Colour::rgb(255, 0, 0), Colour::rgb(0, 255, 0)]];
        let rendered = RenderedShape::new("test", pixels);

        let buffer = rendered.to_rgba_buffer();
        assert_eq!(buffer.len(), 8); // 2 pixels * 4 bytes
        assert_eq!(&buffer[0..4], &[255, 0, 0, 255]); // Red, opaque
        assert_eq!(&buffer[4..8], &[0, 255, 0, 255]); // Green, opaque
    }

    #[test]
    fn test_render_builtin_glyphs() {
        let palette = default_palette();
        let renderer = ShapeRenderer::new(&palette);

        let shape = Shape::new(
            "test",
            vec![],
            vec![vec!['+', '-', '|', '#', '.', 'x']],
            HashMap::new(),
        );

        let rendered = renderer.render(&shape);

        // All builtin glyphs should render without magenta
        assert_eq!(rendered.get(0, 0), Some(Colour::BLACK)); // +
        assert_eq!(rendered.get(1, 0), Some(Colour::BLACK)); // -
        assert_eq!(rendered.get(2, 0), Some(Colour::BLACK)); // |
        assert_eq!(rendered.get(3, 0), Some(Colour::BLACK)); // #
        assert_eq!(rendered.get(4, 0), Some(Colour::WHITE)); // .
        assert_eq!(rendered.get(5, 0), Some(Colour::TRANSPARENT)); // x
    }

    #[test]
    fn test_render_space_glyph() {
        let palette = default_palette();
        let renderer = ShapeRenderer::new(&palette);

        let shape = Shape::new("test", vec![], vec![vec![' ']], HashMap::new());

        let rendered = renderer.render(&shape);

        // Space should render as fill colour
        assert_eq!(rendered.get(0, 0), Some(Colour::WHITE));
    }

    #[test]
    fn test_render_with_legend() {
        let palette = default_palette();
        let renderer = ShapeRenderer::new(&palette);

        let mut legend = HashMap::new();
        legend.insert('B', LegendEntry::StampRef("solid".to_string()));

        let shape = Shape::new("test", vec![], vec![vec!['B', 'B']], legend);

        let rendered = renderer.render(&shape);

        // 'B' mapped to 'solid' stamp (edge colour = black)
        assert_eq!(rendered.get(0, 0), Some(Colour::BLACK));
        assert_eq!(rendered.get(1, 0), Some(Colour::BLACK));
    }

    #[test]
    fn test_render_missing_glyph() {
        let palette = default_palette();
        let renderer = ShapeRenderer::new(&palette);

        let shape = Shape::new("test", vec![], vec![vec!['?']], HashMap::new());

        let rendered = renderer.render(&shape);

        // Unknown glyph should render as magenta
        assert_eq!(rendered.get(0, 0), Some(Colour::MAGENTA));
    }

    #[test]
    fn test_render_with_custom_stamp() {
        let palette = default_palette();
        let custom_stamp = Stamp::single("custom", Some('C'), PixelToken::Fill);

        let mut renderer = ShapeRenderer::new(&palette);
        renderer.add_stamp(&custom_stamp);

        let shape = Shape::new("test", vec![], vec![vec!['C']], HashMap::new());

        let rendered = renderer.render(&shape);

        // 'C' should use custom stamp (fill colour = white)
        assert_eq!(rendered.get(0, 0), Some(Colour::WHITE));
    }

    #[test]
    fn test_render_with_brush_fill() {
        let palette = default_palette();
        let checker = BuiltinBrushes::get("checker").unwrap();

        let mut renderer = ShapeRenderer::new(&palette);
        renderer.add_brush(&checker);

        let mut legend = HashMap::new();
        legend.insert(
            '~',
            LegendEntry::Fill {
                name: "checker".to_string(),
                bindings: [('A', "$edge".to_string()), ('B', "$fill".to_string())]
                    .into_iter()
                    .collect(),
            },
        );

        let shape = Shape::new(
            "test",
            vec![],
            vec![vec!['~', '~'], vec!['~', '~']],
            legend,
        );

        let rendered = renderer.render(&shape);

        // Checker pattern should tile: AB/BA
        assert_eq!(rendered.get(0, 0), Some(Colour::BLACK)); // A = edge
        assert_eq!(rendered.get(1, 0), Some(Colour::WHITE)); // B = fill
        assert_eq!(rendered.get(0, 1), Some(Colour::WHITE)); // B = fill
        assert_eq!(rendered.get(1, 1), Some(Colour::BLACK)); // A = edge
    }

    #[test]
    fn test_render_with_palette_variant() {
        // Create a palette with variants
        use crate::types::PaletteBuilder;

        let mut builder = PaletteBuilder::new("test");
        builder.define("edge", "#000000");
        builder.define("fill", "#FFFFFF");
        builder.define_variant("inverted", "edge", "#FFFFFF");
        builder.define_variant("inverted", "fill", "#000000");

        let palette = builder.build(None).unwrap();

        // Render without variant
        let renderer = ShapeRenderer::new(&palette);
        let shape = Shape::new("test", vec![], vec![vec!['+', '.']], HashMap::new());
        let rendered = renderer.render(&shape);
        assert_eq!(rendered.get(0, 0), Some(Colour::BLACK)); // edge
        assert_eq!(rendered.get(1, 0), Some(Colour::WHITE)); // fill

        // Render with variant
        let renderer = ShapeRenderer::new(&palette).with_variant("inverted");
        let rendered = renderer.render(&shape);
        assert_eq!(rendered.get(0, 0), Some(Colour::WHITE)); // inverted edge
        assert_eq!(rendered.get(1, 0), Some(Colour::BLACK)); // inverted fill
    }

    #[test]
    fn test_render_legend_overrides_glyph() {
        let palette = default_palette();
        let renderer = ShapeRenderer::new(&palette);

        // '+' normally means edge, but legend overrides to fill
        let mut legend = HashMap::new();
        legend.insert('+', LegendEntry::StampRef("fill".to_string()));

        let shape = Shape::new("test", vec![], vec![vec!['+']], legend);

        let rendered = renderer.render(&shape);

        // Should use legend override (fill = white), not builtin (edge = black)
        assert_eq!(rendered.get(0, 0), Some(Colour::WHITE));
    }
}
