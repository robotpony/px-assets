# Changelog

All notable changes to this project will be documented in this file.

## [0.9.0] - 2026-02-03

### Added

- PNG output for rendered shapes
  - `write_png()` function for writing shapes to PNG files
  - `scale_pixels()` for nearest-neighbour integer upscaling
  - `--scale` CLI argument for output scaling (default: 1)
  - `scale:` frontmatter option in shape files (CLI overrides if > 1)
- Functional `px build` command
  - Processes `.shape.md` files
  - Creates output directory automatically
  - Supports `--shader` flag for shader selection
  - Supports `--output` / `-o` for output directory
  - Supports `--scale` for integer upscaling

## [0.8.0] - 2026-02-03

### Added

- Shape renderer for `.shape.md` files
  - `Shape` type with name, tags, ASCII grid, and legend
  - `LegendEntry` enum for stamp refs, brush refs, and fills
  - Shape parser supporting multi-shape files
  - `ShapeRenderer` for converting shapes to pixel grids
  - `RenderedShape` with pixel buffer and RGBA export
  - Glyph resolution order: legend → stamp glyph → builtins → magenta fallback
  - Support for brush fills with coordinate-based tiling
  - Palette variant support in rendering
  - `to_rgba_buffer()` for image output preparation
  - Tags parsed from frontmatter (space-separated with `#` prefix)

## [0.7.0] - 2026-02-03

### Added

- Shader loader for `.shader.md` files
  - `Shader` type for palette binding and effects configuration
  - `ShaderBuilder` for constructing shaders from parsed definitions
  - `Effect` enum with built-in effect types:
    - `vignette` - darkens image edges
    - `scanlines` - adds horizontal scan lines
    - `brightness` - adjusts overall brightness
    - `contrast` - adjusts contrast
    - `Custom` - extensible for unknown effect types
  - `EffectParam` for storing effect parameters
  - Shader inheritance support (child shaders can inherit from parents)
  - `BuiltinShaders` with default shader
  - Effects parsed from YAML but not yet applied (rendering in Phase 1.7+)

## [0.6.0] - 2026-02-03

### Added

- Brush loader for `.brush.md` files
  - `Brush` type with name and pattern grid
  - Positional colour tokens (A, B, C, etc.) bound at usage time
  - Brush parser supporting multi-brush files
  - `BuiltinBrushes` with 7 default brushes:
    - `solid` - 1x1 single colour
    - `checker` - 2x2 checkerboard
    - `diagonal-r` - diagonal lines (/)
    - `diagonal-l` - diagonal lines (\)
    - `h-line` - horizontal stripes
    - `v-line` - vertical stripes
    - `noise` - 4x4 pseudo-random pattern
  - `Brush::render()` for converting pattern with colour bindings
  - `Brush::fill()` for tiling patterns across regions
  - Unbound tokens default to transparent

## [0.5.0] - 2026-02-03

### Added

- Stamp loader for `.stamp.md` files
  - `Stamp` type with name, glyph, and pixel grid
  - `PixelToken` enum for semantic pixels (Edge, Fill, Transparent)
  - Stamp parser supporting multi-stamp files
  - `BuiltinStamps` with 7 default stamps:
    - `corner` (`+`) - edge pixel
    - `edge-h` (`-`) - edge pixel
    - `edge-v` (`|`) - edge pixel
    - `solid` (`#`) - edge pixel
    - `fill` (`.`) - fill pixel
    - `transparent` (`x`) - transparent pixel
    - `space` (` `) - fill pixel
  - `Stamp::render()` for converting tokens to colours
  - Support for variable-size stamps with pixel grid normalization

## [0.4.0] - 2026-02-03

### Added

- Colour expression system for palette definitions
  - `ColourExpr` parser for function-style expressions
  - `ExprEvaluator` for evaluating expressions against a palette
  - `darken($colour, percent)` - reduce lightness in HSL space
  - `lighten($colour, percent)` - increase lightness in HSL space
  - `saturate($colour, percent)` - increase saturation
  - `desaturate($colour, percent)` - decrease saturation
  - `mix($colour1, $colour2, percent)` - blend two colours
  - `alpha($colour, percent)` - set alpha channel
  - Support for nested expressions: `darken(lighten($gold, 10%), 5%)`

### Changed

- `PaletteBuilder::define()` now parses colour expressions automatically

### Dependencies

- Added `palette` crate (0.7) for HSL colour space conversions

## [0.3.0] - 2026-02-02

### Added

- Palette loader for `.palette.md` files
  - `Colour` type with hex parsing (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
  - `Palette` type with named colours and variants
  - Colour reference resolution (`$edge: $dark`) with cycle detection
  - Variant support (`@variant-name:` blocks)
  - Palette inheritance (`inherits: parent-palette`)
  - Builtin default palette with $black, $white, $edge, $fill
- `PaletteBuilder` for constructing palettes from parsed definitions

## [0.2.0] - 2026-02-02

### Added

- Parser infrastructure for markdown-style definition files
  - Document splitter for multi-definition files
  - YAML frontmatter extraction with source span tracking
  - Code block body extraction (```px fences)
  - Legend section parsing (glyph to reference mappings)
  - Source location tracking for error messages
- `Span`, `Spanned<T>`, and `Location` types for source positions
- `RawDocument` type for parsed documents
- `LegendValue` enum for simple and complex legend entries

## [0.1.0] - 2026-02-02

### Added

- Initial project structure
- CLI skeleton with `px build` and `px validate` commands (stubs)
- Error handling with miette diagnostics
- Test fixtures linked to examples
