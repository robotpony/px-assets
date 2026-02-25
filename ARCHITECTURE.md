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

**Primitives** (pixel-level definitions):

| Type | Purpose | Tokens |
|------|---------|--------|
| Palette | Named colours + variants + expressions | `$name: #hex` |
| Brush | Tiling pattern | Positional: `A`, `B`, `C` |
| Stamp | Pixel art with default glyph | Semantic: `$`, `.`, `x` |

**Key distinction**: Stamps use semantic tokens bound to palette (`$edge`, `$fill`). Brushes use positional tokens bound at usage time (`A: $dark, B: $light`).

**Composition** (layout definitions):

| Type | Purpose | Legend Maps To |
|------|---------|----------------|
| Shape | ASCII art using stamps | Stamps (char → stamp) |
| Prefab | Grid of shapes/prefabs | Shapes/Prefabs |
| Map | Level layout (semantic prefab) | Shapes/Prefabs |

All three use **unified legend syntax** for character mappings.

**Rendering** (output configuration):

| Type | Purpose |
|------|---------|
| Shader | Palette binding + lighting + effects |
| Target | Output format, scale, sheet size |

### Dependency Graph

```
Palette ────────────────→ Shader ─────────────────┐
                                                  │
Brush (pattern) ────┐                             │
                    ├──→ Shape ──┬──→ Prefab ─────┼──→ Renderer
Stamp (pixel art) ──┘            │                │
                                 └──→ Map ────────┘
                                       │
                                     Target
```

- **Brush** defines tiling patterns (positional tokens)
- **Stamp** defines pixel art with default glyph (semantic tokens)
- **Shape** uses stamps/brushes (via legend + stamp glyph defaults)
- **Prefab/Map** compose shapes (via legend)
- **Shader** applies palette + effects
- **Target** controls output format

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
    shaders: HashMap<String, Shader>,
    shapes: HashMap<String, Shape>,
    prefabs: HashMap<String, Prefab>,
    maps: HashMap<String, Map>,
    targets: HashMap<String, Target>,
}
```

Validation:
- Resolve all references (brush → stamps, shader → palette, etc.)
- Apply inheritance chains
- Warn on missing refs (substitute placeholders)
- Compute colour expressions

### 4. Render Phase

```rust
// Core rendering pipeline
fn render(shape: &Shape, shader: &Shader, registry: &AssetRegistry) -> RenderResult {
    let resolved = resolve_glyphs(&shape.grid, &shape.legend, registry);  // char → Stamp/Brush
    let pixels = expand_to_pixels(resolved);                               // stamps → pixels
    let colored = apply_shader(pixels, shader);                            // tokens → RGBA
    RenderResult { pixels: colored, metadata: ... }
}
```

**Glyph resolution** uses the registry to find stamps/brushes:
1. Check shape's legend for local override
2. Check stamps for matching `glyph:` declaration
3. Fall back to builtin stamps

**Stamp sizing** is controlled by target:
- Target declares `tile: 8x8` (or omit for native size)
- Stamps pad (centre) or clip to fit during output

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

### Glyph Resolver

```rust
struct GlyphResolver {
    stamps: HashMap<String, Stamp>,
    brushes: HashMap<String, Brush>,
    builtins: HashMap<char, Stamp>,
}

impl GlyphResolver {
    /// Resolve a glyph to a stamp/brush using resolution order:
    /// 1. Shape's legend (passed in)
    /// 2. Stamp's declared glyph
    /// 3. Builtin defaults
    fn resolve(&self, glyph: char, legend: &Legend) -> ResolvedGlyph;
}

enum ResolvedGlyph {
    Stamp(Stamp),
    Brush { pattern: Brush, colors: HashMap<char, ColorRef> },
    Fill { pattern: Brush, colors: HashMap<char, ColorRef> },
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
    MissingStamp { glyph: char, shape: String, location: Location },
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
│   ├── slice.rs      # px slice
│   ├── watch.rs      # px watch
│   ├── validate.rs   # px validate
│   └── preview.rs    # px preview
├── parser/
│   ├── mod.rs
│   ├── frontmatter.rs
│   ├── palette.rs
│   ├── stamp.rs
│   ├── brush.rs
│   ├── shader.rs
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
├── slice/
│   ├── mod.rs         # Orchestrates the reverse pipeline
│   ├── grid.rs        # Grid detection and cell slicing
│   ├── palette.rs     # Colour extraction (shared with cli/palette.rs)
│   ├── stamps.rs      # Structural hashing and stamp detection
│   └── generate.rs    # .shape.md / .stamp.md / .palette.md writers
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
  - brushes/
  - shaders/
  - shapes/
  - prefabs/
  - maps/
  - targets/

defaults:
  brush: default
  shader: default
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
Input:        +--+        Legend:
              |BB|        B: brick
              |BB|
              +--+

1. Tokenize:  ['+', '-', '-', '+']
              ['|', 'B', 'B', '|']
              ['|', 'B', 'B', '|']
              ['+', '-', '-', '+']

2. Resolve:   Legend: B → brick (local override)
              Builtin: +, -, | → corner, edge-h, edge-v

              [corner, edge-h, edge-h, corner]
              [edge-v, brick,  brick,  edge-v]
              [edge-v, brick,  brick,  edge-v]
              [corner, edge-h, edge-h, corner]

3. Expand:    Each stamp expands to its pixel grid
              (1x1 builtins = 1px, 8x4 brick = 8x4px)

4. Colorize:  $ → palette.$edge
              . → palette.$fill
              x → transparent

Output:       Final RGBA pixel buffer
```

For brush fills (e.g., `~: { fill: checker, A: $edge, B: $fill }`):

```
1. Resolve glyph to brush + colour binding
2. Tile the brush pattern to fill the cell
3. Map A/B tokens to bound colours
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

## Reverse Pipeline (`px slice`)

The forward pipeline builds images from definitions. The reverse pipeline recovers definitions from existing images.

```
Input PNG → Read Pixels → Slice Grid → Analyse → Detect Stamps → Generate Files
    ↓           ↓             ↓           ↓            ↓              ↓
  .png      RgbaImage    Vec<Cell>    Palette     Stamp groups    .palette.md
                                      + colour    + structural    .stamp.md
                                      frequency   hashes          .shape.md
```

### Phase 1: Load & Slice

```
Input PNG
    │
    ├── --cell WxH provided ──→ Uniform grid split
    │
    └── No --cell ──→ Auto-detect separators
                          │
                          ├── Found grid ──→ Split at detected boundaries
                          └── No grid ──→ Treat as single sprite

    Then: skip fully transparent cells
```

**Auto-detection algorithm:**
1. For each row, check if all pixels match the separator colour (or all transparent)
2. Same for columns
3. `--separator` overrides which colour counts as a separator (default: transparent, then auto-detect the most common uniform-row colour)
4. Find most common spacing between consecutive separator rows/columns
5. If consistent spacing found → use as cell dimensions
6. Handle multi-pixel-thick separators by collapsing adjacent separator rows/cols

Each cell becomes a `SlicedCell`:

```rust
struct SlicedCell {
    name: String,        // e.g., "sprites-0-2" (row-col)
    pixels: RgbaImage,   // Cropped pixel data
    origin: (u32, u32),  // Position in source image
}
```

Fully transparent cells (every pixel has alpha = 0) are skipped with a note to stderr.

### Phase 2: Palette Extraction

Collect all unique RGBA values across all cells. Skip fully transparent pixels.

```rust
struct ExtractedPalette {
    colours: Vec<(String, Rgba)>,  // ($colour-0, #hex), sorted by frequency
    lookup: HashMap<Rgba, String>, // Reverse map: pixel → palette name
}
```

Output: one `.palette.md` with `$colour-N: #RRGGBB` lines.

Builds on the existing `px palette` command logic but structured for downstream consumption rather than stdout.

### Phase 3: Stamp Detection

Stamps are detected by finding repeating rectangular sub-blocks within and across cells.

**Block size selection:** Try all sizes that evenly divide the cell dimensions. For a 16x16 cell, test 2x2, 4x4, 8x8, 16x16. Pick the size that maximizes reuse (most repeated blocks).

**Structural hashing** enables colour-variant detection:

```
Block A: [Red, Red, Blue, Blue]  → Structure: [0, 0, 1, 1]
Block B: [Grn, Grn, Ylw, Ylw]   → Structure: [0, 0, 1, 1]
                                    ↑ Same structure → colour variants
```

Algorithm:
1. For each NxN block, replace colours with order-of-first-appearance indices
2. The index grid IS the structural hash
3. Group blocks by structural hash
4. Groups with 2+ instances → stamp definitions
5. Map structural positions (0, 1, 2...) to positional tokens (`A`, `B`, `C`... following the brush convention)

```rust
struct DetectedStamp {
    name: String,                       // e.g., "stamp-a"
    glyph: char,                        // Assigned glyph for shapes
    structure: Vec<Vec<u8>>,            // Structural pattern (index grid)
    variants: Vec<HashMap<u8, Rgba>>,   // Colour bindings per variant
}
```

**Glyph assignment** for shape grids:
1. Reserve builtins: `+`, `-`, `|`, `#`, `.`, `x`, ` `
2. Uppercase A-Z (26 glyphs)
3. Lowercase a-z excluding x (25 glyphs)
4. Digits 0-9 (10 glyphs)
5. Symbols: `@`, `!`, `?`, `~`, `*`, `^`, `&`, `%`
6. Warn if colour/stamp count exceeds available glyphs (~70)

### Phase 4: Shape Generation

For each sliced cell, produce a `.shape.md`:

- If stamp detection succeeded: shape grid at stamp-level resolution (cell_width / stamp_width)
- If no stamps: shape grid at pixel-level resolution (one glyph per pixel)
- Legend maps each glyph to either a stamp name or a palette colour
- Transparent pixels → `x` glyph

```
Generated shape (pixel-level, no stamps):

---
name: sprites-0-0
---

```px
ABB
ACC
ABB
```

---
A: { stamp: solid, $fill: $colour-0 }
B: { stamp: solid, $fill: $colour-1 }
C: { stamp: solid, $fill: $colour-2 }
```

### Slice Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Block size selection | Derive from cell size | Avoids user guesswork; tries all divisors |
| Structural hashing | First-appearance index | Simple, fast, handles arbitrary colour counts |
| Palette scope | One shared palette per slice run | Colours often shared across sprites in a sheet |
| Stamp scope | Global across all cells | Maximizes reuse detection |
| Glyph assignment | Frequency-based (most common first) | Common glyphs get easy-to-read characters |
| Grid auto-detect | Separator row/column scanning | Works for most sprite sheet conventions |
| `--stamp-size` validation | Must evenly divide cell dims | Warn and fall back to auto if it doesn't divide evenly |
| Fallback | Always produce output | Pixel-level shapes work even when detection fails |

## Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Error handling | Warn and continue | Better UX during iteration; see all issues at once |
| Stamp sizing | Configurable per brush | Balance between simplicity and flexibility |
| Colour expressions | Full support | Enables sophisticated palette variations |
| Cache strategy | Content-hash based | More reliable than mtime for CI/CD |
| Parallelism | Rayon work-stealing | Good default for mixed workloads |
| Plugin system | None | Keep core simple; features via releases |
| Output formats | PNG, P8, Aseprite | Cover major use cases without sprawl |
| Builtins | Load from known path | `~/.px/defaults/` with embedded fallback; projects can override via `inherits:` |
