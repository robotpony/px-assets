# px Architecture

## Overview

px transforms markdown-style definition files into sprites and sprite maps. The pipeline flows from human-readable ASCII definitions through a layered rendering system to platform-specific outputs.

```
Source Files → Parser → Asset Registry → Renderer → Output Writers
     ↓              ↓           ↓            ↓            ↓
  .palette.md    YAML+Body   Validated    Rasterized   PNG/P8/Aseprite
  .stamp.md      extraction   refs        pixels
  .shape.md
  ...
```

## Core Concepts

### Asset Types

| Type | Purpose | Dependencies |
|------|---------|--------------|
| Palette | Named colours + variants + expressions | None |
| Brush | Fill patterns (tiling) | None |
| Stamp | Glyph → pixel grid mapping | None |
| Style | Binds palette + stamps + grid config | Palette, Stamps, Brushes |
| Shape | ASCII composition | None (resolved via Style) |
| Prefab | Shape/prefab placement grid | Shapes, other Prefabs |
| Map | Level layout (semantic prefab) | Shapes, Prefabs |
| Target | Output configuration | None |

### Dependency Graph

```
Palette ─────────────────────┐
                             │
Brush ───────────────────────┼──→ Style ──→ Renderer
                             │        ↑
Stamp ───────────────────────┘        │
                                      │
Shape ────────────────────────────────┤
                                      │
Prefab ──→ (contains Shapes/Prefabs) ─┤
                                      │
Map ─────→ (contains Shapes/Prefabs) ─┘
```

## Data Flow

### 1. Discovery Phase

```
Project Root
    ├── px.yaml (optional manifest)
    ├── palettes/
    │   └── *.palette.md
    ├── stamps/
    │   └── *.stamp.md
    └── ...
```

- Scan directories for files by extension
- Optional `px.yaml` manifest overrides conventions
- Build source file index with modification times (for cache)

### 2. Parse Phase

Each file is parsed into:
```rust
struct ParsedFile {
    path: PathBuf,
    mtime: SystemTime,
    documents: Vec<Document>,  // Multiple per file, separated by ---
}

struct Document {
    frontmatter: HashMap<String, Value>,  // YAML
    body: String,                          // Raw content
    legend: Option<HashMap<char, String>>, // For prefab/map
}
```

### 3. Registry Phase

Parsed documents become typed assets in the registry:

```rust
struct AssetRegistry {
    palettes: HashMap<String, Palette>,
    brushes: HashMap<String, Brush>,
    stamps: HashMap<String, Stamp>,
    styles: HashMap<String, Style>,
    shapes: HashMap<String, Shape>,
    prefabs: HashMap<String, Prefab>,
    maps: HashMap<String, Map>,
    targets: HashMap<String, Target>,
}
```

Validation:
- Resolve all references (palette → colours, style → stamps, etc.)
- Apply inheritance chains
- Warn on missing refs (substitute placeholders)
- Compute colour expressions

### 4. Render Phase

```rust
// Core rendering pipeline
fn render(shape: &Shape, style: &Style, registry: &AssetRegistry) -> RenderResult {
    let grid = resolve_stamps(&shape.grid, style);  // char → Stamp
    let pixels = expand_stamps(grid, style);         // stamps → pixels
    let colored = apply_palette(pixels, style);      // tokens → RGBA
    RenderResult { pixels: colored, metadata: ... }
}
```

**Stamp sizing** is configurable per style:
- Style declares `grid_size: 8x8` (or auto)
- Stamps pad (centre) or clip to fit grid
- Variable-size stamps supported when `grid_size: auto`

### 5. Output Phase

Renderers produce intermediate `RenderResult`, then output writers convert to target format:

```rust
trait OutputWriter {
    fn write(&self, result: &RenderResult, target: &Target) -> Result<()>;
}

// Implementations
struct PngWriter;      // PNG with optional JSON metadata
struct Pico8Writer;    // .p8 format with indexed colours
struct AsepriteWriter; // .aseprite with layers
```

## Key Components

### Parser

```rust
mod parser {
    pub fn parse_file(path: &Path) -> Result<ParsedFile>;
    pub fn split_documents(content: &str) -> Vec<&str>;
    pub fn extract_frontmatter(doc: &str) -> (Frontmatter, Body);
    pub fn extract_legend(body: &str) -> (Content, Option<Legend>);
}
```

### Colour Engine

Full expression support for palette definitions:

```rust
enum ColorExpr {
    Hex(String),                      // #ff0000
    Reference(String),                // $other-color
    Darken(Box<ColorExpr>, f32),      // darken($fill, 20%)
    Lighten(Box<ColorExpr>, f32),     // lighten($edge, 10%)
    Saturate(Box<ColorExpr>, f32),    // saturate($mid, 15%)
    Desaturate(Box<ColorExpr>, f32),  // desaturate($mid, 15%)
    Shift(Box<ColorExpr>, f32),       // shift($fill, 30) - hue rotation
    Mix(Box<ColorExpr>, Box<ColorExpr>, f32), // mix($a, $b, 50%)
    Alpha(Box<ColorExpr>, f32),       // alpha($fill, 0.5)
}

fn evaluate(expr: &ColorExpr, palette: &Palette) -> Rgba;
```

### Stamp Resolver

```rust
struct StampResolver {
    style: Style,
    stamps: HashMap<String, Stamp>,
    grid_size: Option<(u32, u32)>,  // None = variable
}

impl StampResolver {
    fn resolve(&self, glyph: char) -> ResolvedStamp;
    fn pad_to_grid(&self, stamp: &Stamp) -> Stamp;
}
```

### Cache System

Persistent cache for incremental builds:

```rust
struct BuildCache {
    file_hashes: HashMap<PathBuf, u64>,
    rendered: HashMap<String, CachedRender>,
}

impl BuildCache {
    fn load(path: &Path) -> Self;
    fn save(&self, path: &Path);
    fn is_stale(&self, asset: &str, deps: &[&str]) -> bool;
}
```

Cache invalidation:
- Track file content hashes (not just mtime)
- Track dependency graph
- Invalidate downstream assets when upstream changes

## Error Handling

**Philosophy**: Warn and continue. Produce output with placeholders for missing references.

```rust
struct BuildContext {
    warnings: Vec<Warning>,
    errors: Vec<Error>,
}

enum Warning {
    MissingStamp { glyph: char, style: String, location: Location },
    MissingColour { name: String, palette: String, location: Location },
    StampSizeMismatch { stamp: String, expected: (u32, u32), actual: (u32, u32) },
    // ...
}
```

- Missing stamp → render magenta placeholder
- Missing colour → render magenta
- Invalid expression → use fallback, warn
- All warnings collected and reported after build

## Concurrency

```rust
// Parallel rendering where dependency graph allows
fn build_parallel(registry: &AssetRegistry) -> Vec<RenderResult> {
    // Group assets by dependency depth
    // Render each depth level in parallel
    // Use rayon for work-stealing parallelism
}
```

## Module Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library root
├── cli/
│   ├── mod.rs
│   ├── build.rs      # px build
│   ├── watch.rs      # px watch
│   ├── validate.rs   # px validate
│   └── preview.rs    # px preview
├── parser/
│   ├── mod.rs
│   ├── frontmatter.rs
│   ├── palette.rs
│   ├── stamp.rs
│   ├── style.rs
│   ├── shape.rs
│   ├── prefab.rs
│   ├── map.rs
│   └── target.rs
├── registry/
│   ├── mod.rs
│   ├── resolver.rs   # Reference resolution
│   └── validator.rs  # Validation passes
├── colour/
│   ├── mod.rs
│   ├── expr.rs       # Expression parser
│   ├── eval.rs       # Expression evaluator
│   └── convert.rs    # RGB/HSL/etc conversions
├── render/
│   ├── mod.rs
│   ├── stamp.rs      # Stamp expansion
│   ├── shape.rs      # Shape rendering
│   ├── composite.rs  # Prefab/map composition
│   └── sheet.rs      # Sprite sheet packing
├── output/
│   ├── mod.rs
│   ├── png.rs
│   ├── pico8.rs
│   ├── aseprite.rs
│   └── metadata.rs   # JSON export
├── cache/
│   ├── mod.rs
│   └── hash.rs
└── server/
    ├── mod.rs
    └── hot_reload.rs
```

## Configuration

### Project Manifest (optional)

```yaml
# px.yaml
sources:
  - palettes/
  - stamps/
  - styles/
  - shapes/
  - prefabs/
  - maps/
  - targets/

defaults:
  style: default
  target: web

output:
  dir: dist/
  clean: true

cache:
  dir: .px-cache/
  enabled: true
```

### Target Configuration

```yaml
# web.target.md
---
name: web
format: png
---
scale: 4
sheet: auto
metadata: true
```

```yaml
# pico8.target.md
---
name: pico8
format: p8
---
sheet: 128x128
tile: 8x8
colors: 16
palette_mode: indexed
dither: ordered
```

## Rendering Pipeline Detail

### Shape → Pixels

```
Input:        +--+
              |BB|
              |BB|
              +--+

1. Tokenize:  ['+', '-', '-', '+']
              ['|', 'B', 'B', '|']
              ['|', 'B', 'B', '|']
              ['+', '-', '-', '+']

2. Resolve:   [corner, edge-h, edge-h, corner]
              [edge-v, brick,  brick,  edge-v]
              [edge-v, brick,  brick,  edge-v]
              [corner, edge-h, edge-h, corner]

3. Expand:    Each stamp expands to its pixel grid
              (1x1 stamps = 1px, 8x8 brick = 8x8px)

4. Colorize:  $ → palette.$edge
              . → palette.$fill
              x → transparent
              A/B → brush pattern colours

Output:       Final RGBA pixel buffer
```

### Prefab/Map Composition

```
Input (prefab):    C
                   W
                   W
                   B

Legend:            C: tower-cap
                   W: wall-segment
                   B: tower-base

1. Parse grid positions
2. Recursively render each referenced shape/prefab
3. Composite at calculated offsets
4. Export combined image + metadata (positions, tags)
```

## Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Error handling | Warn and continue | Better UX during iteration; see all issues at once |
| Stamp sizing | Configurable per style | Balance between simplicity and flexibility |
| Colour expressions | Full support | Enables sophisticated palette variations |
| Cache strategy | Content-hash based | More reliable than mtime for CI/CD |
| Parallelism | Rayon work-stealing | Good default for mixed workloads |
| Plugin system | None | Keep core simple; features via releases |
| Output formats | PNG, P8, Aseprite | Cover major use cases without sprawl |
