# Changelog

All notable changes to this project will be documented in this file.

## [0.18.1] - 2026-02-09

### Fixed

- Legend parser now skips HTML comments between code block and `---` delimiter
  - Maps (and shapes/prefabs) with HTML comments before the legend section were silently parsed with no legend, producing empty 1x1-cell renders
  - Affects any `.map.md`, `.shape.md`, or `.prefab.md` file using markdown comments for documentation

## [0.18.0] - 2026-02-09

### Added

- Directory-aware build (Phase 2.8)
  - `px build` with no arguments scans current directory, reads `px.yaml` if present
  - `px build shapes/ prefabs/` scans directories for assets
  - `px build shapes/*.shape.md` still works (explicit files)
  - Discovery summary printed to stderr when scanning: "Discovered N shapes, N prefabs, N maps"
  - Manifest settings merged into build: CLI > target > manifest > frontmatter > defaults
  - Manifest `output` used as fallback when `-o` not specified
  - Manifest `scale` and `shader` merged into the settings chain
- `px init` command
  - Scans directory for assets and generates `px.yaml` manifest
  - Discovers source directories from asset file locations
  - `--force` flag to overwrite existing `px.yaml`
  - Optional path argument (default: current directory)

### Changed

- `px build` no longer requires file arguments (defaults to current directory scan)
- `-o` / `--output` is now optional (defaults to manifest `output` or "dist")

## [0.17.0] - 2026-02-09

### Added

- Target system for named output profiles (Phase 2.7)
  - `Target` type bundling format, scale, sheet config, padding, palette mode, and shader
  - `TargetBuilder` for constructing targets from parsed definitions
  - `SheetConfig` enum: `None`, `Auto`, `Fixed { width, height }` with `parse()` for "auto", "WxH", "none"
  - `PaletteMode` enum: `Rgba`, `Indexed` (indexed reserved for Phase 3 formats)
  - `BuiltinTargets` with two profiles:
    - `web`: PNG, no sheet, all defaults
    - `sheet`: PNG, auto sheet packing
  - `.target.md` parser with frontmatter + body key-value support (unknown keys ignored for forward compat)
  - `--target` CLI flag: `--target=web`, `--target=sheet`, or `--target=path/to/custom.target.md`
  - Target resolution: builtins checked first, then file path, else error with help text
  - Setting merge order: CLI flags > target profile > per-asset frontmatter > defaults
  - `check_target_format()` validation: warns if target uses unsupported format (only PNG for now)
  - Registry support: `AssetKind::Target`, `AssetId::target()`, target storage and accessors
  - Discovery support: `.target.md` file detection, scanning, and loading

### Changed

- `--scale` changed from `u32` (default 1) to `Option<u32>` (no default)
  - `None` means "use target scale, then frontmatter scale, then 1"
  - `Some(n)` with n > 1 overrides all other scale sources
- `--padding` changed from `u32` (default 0) to `Option<u32>` (no default)
  - Follows same merge chain: CLI > target > 0
- `--shader` now merges with target shader: CLI > target shader > "default"
- `--sheet` flag now composes with target sheet config: CLI flag activates Auto, target can set Auto or Fixed
- Internal: process functions now accept explicit `default_scale` and `output` params instead of `&BuildArgs`

### Fixed

- Sheet JSON frame coordinates now match actual scaled PNG pixel dimensions (were at 1x regardless of `--scale`)
- Sheet JSON `meta.version` now uses crate version instead of hardcoded "0.16.0"
- Example files: tags now quoted in YAML frontmatter (`tags: "#player"`) so they're parsed correctly instead of treated as YAML comments

## [0.16.0] - 2026-02-09

### Added

- Metadata export for all asset types (Phase 2.6)
  - Shape metadata: `{name}.json` alongside each shape PNG with name, pixel size, and tags
  - Prefab metadata: `{name}.json` alongside each prefab PNG with name, pixel size, tags, grid dimensions, cell size, and instance positions (which shapes are placed where)
  - `ShapeMetadata` struct in types module (derives `Serialize`)
  - `PrefabMetadata` and `PrefabInstance` structs in types module (derive `Serialize`)
  - `PrefabRenderer::render()` now returns `(RenderedShape, PrefabMetadata)` tuple with instance tracking
  - `write_metadata_json()` helper in build command for consistent JSON output
  - All asset types (shapes, prefabs, maps, sheets) now export JSON metadata alongside PNGs

## [0.15.0] - 2026-02-07

### Added

- Sprite sheet packer (Phase 2.5)
  - `--sheet` flag on `px build` packs all shapes and prefabs into a single sprite sheet
  - `--padding N` flag adds N pixels between sprites in the sheet (default 0)
  - Shelf packing algorithm: sorts by height, packs left-to-right in rows
  - Auto-sized sheet width (nearest power-of-two) with height growing as needed
  - Outputs `sheet.png` + `sheet.json` in TexturePacker-compatible JSON Hash format
  - `SheetPacker` and `SheetMeta` types in the render module
  - Maps are skipped in sheet mode (they're level layouts, not atlas sprites)

## [0.14.0] - 2026-02-07

### Added

- Validation system for asset registries (Phase 1.9)
  - `Severity`, `Diagnostic`, and `ValidationResult` types for structured diagnostics
  - Nine validation checks covering shapes, prefabs, and maps:
    - `check_empty_grids`: error on zero-size grids
    - `check_duplicate_names`: warn on shape/prefab name collisions
    - `check_shape_legend_refs`: error on missing stamp/brush references
    - `check_prefab_legend_refs`: error on missing shape/prefab references
    - `check_map_legend_refs`: error on missing references (skips "empty")
    - `check_unmapped_glyphs`: warn on grid glyphs with no legend entry and no builtin match
    - `check_unused_legends`: warn on legend entries never used in the grid
    - `check_stamp_sizes`: warn when a shape mixes stamps of different dimensions
    - `check_palette_refs`: warn on brush bindings referencing unknown palette colours
  - `validate_registry()` orchestrator runs all checks and merges results
  - `print_diagnostics()` formats output to stderr with error codes and help text
  - `px validate` command: discovers, loads, and validates assets, exits 1 on errors
  - `px build --validate` flag: runs validation before building, aborts on errors

## [0.13.0] - 2026-02-07

### Added

- Map renderer for level layouts (Phase 2.4)
  - `Map` type with ASCII placement grid and name-reference legend (same structure as Prefab)
  - `parse_map_file()` parser for `.map.md` files
  - `MapRenderer` for compositing shapes/prefabs onto a map canvas
  - `empty` reserved name: legend entries mapping to "empty" produce transparent cells with no metadata
  - `MapMetadata` and `MapInstance` structs for JSON export (derives `Serialize`)
  - JSON metadata output alongside PNG: grid dimensions, cell size, pixel positions per shape
  - Registry integration with dependency tracking (maps depend on shapes/prefabs)
  - Loader integration: `load_assets()` now discovers and loads `.map.md` files
  - Three-phase CLI build: shapes first, then prefabs, then maps
  - Scale support via frontmatter or CLI `--scale` flag

## [0.12.0] - 2026-02-05

### Added

- Prefab renderer for compositing shapes into larger images (Phase 2.3)
  - `Prefab` type with ASCII placement grid and name-reference legend
  - `parse_prefab_file()` parser for `.prefab.md` files
  - Legend validation: rejects complex brush/fill entries (simple name refs only)
  - `PrefabRenderer` for compositing pre-rendered shapes onto a canvas
  - Uniform cell sizing (max width x max height of referenced shapes)
  - Transparency-aware blit (alpha > 0 overwrites, alpha == 0 skips)
  - Nested prefab support (rendered prefabs available for later prefabs)
  - Shared `parse_grid` between shape and prefab parsers
  - Registry integration with dependency tracking (prefabs depend on shapes/prefabs)
  - Topological sort ensures correct render order for nested prefabs
  - Loader integration: `load_assets()` now discovers and loads `.prefab.md` files
  - Two-phase CLI build: shapes render first, then prefabs composite them
  - Scale support via frontmatter or CLI `--scale` flag

## [0.11.0] - 2026-02-03

### Added

- File discovery system for px projects (Phase 2.2)
  - `discover()` function to find all assets in a project directory
  - `px.yaml` manifest support with full configuration:
    - `sources`: directories/globs to scan
    - `output`: output directory
    - `target`: default target name
    - `shader`: default shader name
    - `scale`: default scale factor
    - `excludes`: patterns to exclude
  - `ScanResult` categorizing discovered files by asset type
  - `LoadOptions` for controlling builtin inclusion
  - Automatic asset loading into `RegistryBuilder`
  - Glob pattern matching for excludes (`*.bak`, `**/temp/*`, etc.)
  - Convention-based discovery (scans current dir if no manifest)

## [0.10.0] - 2026-02-03

### Added

- Asset registry for centralized asset management (Phase 2.1)
  - `AssetRegistry` for storing palettes, stamps, brushes, shaders, shapes
  - `AssetId` with kind + name for unique identification (allows same name across types)
  - `AssetKind` enum: Palette, Stamp, Brush, Shader, Shape, Prefab, Map
  - `RegistryBuilder` for constructing registries from parsed assets
  - `DependencyGraph` tracking relationships between assets
  - Topological sort for determining correct build order
  - Cycle detection with path reporting for circular dependencies
  - Immutable registry design (build once)

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
