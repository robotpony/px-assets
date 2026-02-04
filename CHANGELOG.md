# Changelog

All notable changes to this project will be documented in this file.

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
