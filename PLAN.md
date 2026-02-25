# px Implementation Plan

Phased approach from core parsing to full toolchain.

## Phase 1: Foundation

**Goal**: Parse files, render single shapes to PNG.

### 1.1 Project Setup

- [x] Initialize Cargo project with workspace structure
- [x] Configure dependencies (serde, image, clap, thiserror, miette)
- [x] Set up basic CLI skeleton (`px build`, `px validate`)
- [x] Create test fixtures directory with example files

### 1.2 Parser Infrastructure

- [x] Implement document splitter (multi-doc per file)
- [x] Implement frontmatter extractor (YAML between `---`)
- [x] Implement legend extractor (after second `---`)
- [x] Add source location tracking for error messages

### 1.3 Palette Loader

- [x] Parse colour definitions (`$name: #hex`)
- [x] Resolve references (`$edge: $dark`)
- [x] Implement colour expression parser (darken, lighten, etc.)
- [x] Implement colour expression evaluator
- [x] Add HSL conversion (via palette crate)
- [x] Support variants (`@variant-name:` blocks)
- [x] Support inheritance (`inherits:`)

### 1.4 Stamp Loader

- [x] Parse stamp definitions (glyph + pixel grid)
- [x] Implement builtin stamps (corner, edge-h, edge-v, solid, fill, transparent)

### 1.5 Brush Loader

- [x] Parse brush definitions (pattern grid with A/B tokens)
- [x] Implement builtin brushes (solid, checker, diagonal-l/r, h-line, v-line)

### 1.6 Shader Loader

- [x] Parse palette reference
- [x] Parse effects list
- [x] Support inheritance

### 1.7 Shape Renderer

- [x] Parse shape grid (ASCII body)
- [x] Parse shape legend (glyph overrides)
- [x] Implement glyph resolution (legend → stamp glyph → builtins)
- [x] Expand stamps to pixels
- [x] Handle brush fills with colour binding
- [x] Apply palette colours
- [x] Implement placeholder rendering for missing stamps (magenta)

### 1.8 PNG Output

- [x] Render shape to image buffer
- [x] Write PNG file
- [x] Implement scale factor (integer upscaling)

### 1.9 Validation & Warnings

- [x] Collect warnings during build (missing refs, size mismatches)
- [x] Implement miette-style error formatting
- [x] Add `px validate` command

**Deliverable**: `px build shapes/wall.shape.md --brush=default --shader=default` outputs PNG.

---

## Phase 2: Composition

**Goal**: Prefabs, maps, sprite sheets, metadata.

### 2.1 Asset Registry

- [x] Implement centralized asset store
- [x] Build dependency graph
- [x] Topological sort for build order
- [x] Detect circular references

### 2.2 File Discovery

- [x] Implement walkdir-based scanner
- [x] Filter by extension
- [x] Parse px.yaml manifest (optional)
- [x] Merge convention + manifest sources

### 2.3 Prefab Renderer

- [x] Parse prefab grid + legend
- [x] Resolve shape/prefab references
- [x] Calculate layout positions
- [x] Composite rendered shapes
- [x] Support nested prefabs

### 2.4 Map Renderer

- [x] Parse map (same as prefab)
- [x] Handle `empty` reserved name
- [x] Generate instance metadata (positions, cells)

### 2.5 Sprite Sheet Packer

- [x] Implement shelf packing algorithm
- [x] Support auto-sizing (power-of-two width)
- [x] Add configurable padding (`--padding N`)
- [x] Generate frame metadata (x, y, w, h)
- [x] TexturePacker-compatible JSON Hash output

### 2.6 Metadata Export

- [x] Generate shape metadata JSON
- [x] Generate sheet metadata JSON
- [x] Generate map metadata JSON (instances, grid info)
- [x] Generate prefab metadata JSON (instances, grid info)

### 2.7 Target System

- [x] `Target` type bundling format, scale, sheet config, padding, palette mode, and shader
- [x] `TargetBuilder` for constructing targets from parsed definitions
- [x] `SheetConfig` enum (`None`, `Auto`, `Fixed`) with string parsing
- [x] `PaletteMode` enum (`Rgba`, `Indexed`)
- [x] `BuiltinTargets` with `web` and `sheet` profiles
- [x] `.target.md` parser with frontmatter + body key-value support
- [x] `--target` CLI flag: builtins, file paths, or error with help text
- [x] Setting merge order: CLI flags > target profile > per-asset frontmatter > defaults
- [x] `--scale` and `--padding` changed to `Option<u32>` for proper merge chain
- [x] Registry, discovery, and loader integration for `.target.md` files
- [x] `check_target_format()` validation: warns if format is not "png"

### 2.8 Directory-Aware Build & Init

- [x] `px build` with no args scans current directory (reads `px.yaml`)
- [x] `px build <dir>` scans directories for assets
- [x] Manifest settings merge: CLI > target > manifest > frontmatter > defaults
- [x] `-o` fallback to manifest `output`
- [x] `px init` generates `px.yaml` from discovered assets

**Deliverables**:

`px build --target=sheet` outputs sprite sheet + JSON.

`px build --target=web` outputs individual PNGs + JSON metadata.

`px build` (no args) discovers and builds all assets in current directory.

`px init` generates a `px.yaml` manifest.

---

## Phase 3: Additional Targets

**Goal**: PICO-8 and Aseprite output.

### 3.1 PICO-8 Output

- [x] Implement colour quantization (16 colours)
- [x] Implement ordered dithering
- [x] Implement Floyd-Steinberg dithering
- [x] Generate P8 sprite data format
- [x] Write .p8 file
- [x] Map to PICO-8 standard palette

### 3.2 Aseprite Output

- [ ] Implement asefile writer
- [ ] Preserve shape layers
- [ ] Export palette
- [ ] Handle animation frames (future)

### 3.3 Target Improvements

- [ ] Add target-specific validation
- [ ] Warn when colours exceed target limits
- [ ] Implement palette_mode: indexed

**Deliverable**: `px build --target=pico8` outputs .p8 file.

---

## Phase 4: Iteration Tools

**Goal**: Watch mode, preview server, caching.

### 4.1 Cache System

- [ ] Implement content hashing (seahash)
- [ ] Store cache in `.px-cache/`
- [ ] Track file → asset mapping
- [ ] Track dependency graph in cache
- [ ] Implement cache invalidation
- [ ] Load/save cache between runs

### 4.2 Watch Mode

- [ ] Implement file watcher (notify)
- [ ] Debounce changes
- [ ] Determine affected assets
- [ ] Incremental rebuild
- [ ] Report build results to terminal

### 4.3 Preview Server

- [ ] Implement axum HTTP server
- [ ] Serve rendered assets
- [ ] Implement WebSocket for hot reload
- [ ] Build preview HTML UI
- [ ] Implement zoom/pan
- [ ] Implement grid overlay
- [ ] Implement inspector panel
- [ ] Implement variant comparison

### 4.4 Parallel Builds

- [ ] Identify parallelizable work (independent assets)
- [ ] Implement rayon parallel rendering
- [ ] Maintain correct build order for dependencies

**Deliverable**: `px watch --preview` with hot reload.

---

## Phase 5: Polish

**Goal**: Refinement, DX improvements, edge cases.

### 5.1 CLI Enhancements

- [x] Implement `px init`
- [x] Implement `px palette` (extract colours from PNG → `.palette.md` format)
- [x] Implement `px list`
- [x] Add shell completions (clap_complete)
- [x] Add `--verbose` and `--quiet` modes
- [x] Add colour output (terminal colours)

### 5.2 Advanced Validation

- [x] Warn on unused assets
- [x] Warn on shadowed definitions
- [x] Check for palette colour unused in shapes
- [x] Suggest fixes for common errors

### 5.3 Documentation

- [ ] Write user guide
- [ ] Document file format spec (from SPEC.md)
- [ ] Add inline examples in error messages
- [ ] Generate man pages

### 5.4 Testing

- [x] Snapshot tests for rendered output
- [x] Unit tests for parsers
- [x] Integration tests for full pipeline
- [x] Performance benchmarks

**Deliverable**: Production-ready CLI tool.

---

## Phase 6: Slice (Reverse Pipeline)

**Goal**: Import existing PNGs by reverse-engineering them into px definition files.

### 6.1 CLI Skeleton & PNG Loading

- [x] Add `px slice` subcommand with clap derive
- [x] Accept `<input>` path, `--cell`, `--output`, `--name`, `--stamps`, `--stamp-size`, `--separator`, `--palette`
- [x] Load PNG via `image::open`, convert to `RgbaImage`
- [x] Validate input (exists, is PNG, has nonzero dimensions)
- [x] Wire up Cargo-style terminal output (reuse `output.rs` printer)

### 6.2 Grid Slicing (Explicit)

- [ ] Parse `--cell WxH` argument (e.g., `16x16`)
- [ ] Split image into uniform cells, producing `Vec<SlicedCell>`
- [ ] Name cells as `{name}-{row}-{col}` (zero-indexed)
- [ ] Handle edge cells that don't fill a complete cell (warn + include)
- [ ] Skip fully transparent cells

### 6.3 Grid Auto-Detection

- [ ] Scan all rows: mark rows where every pixel is identical (or all transparent)
- [ ] Scan all columns: same logic
- [ ] Collapse adjacent separator rows/columns (multi-pixel separators)
- [ ] Find most common spacing → derive cell dimensions
- [ ] Support `--separator` colour override (default: transparent or auto)
- [ ] Fallback: if no grid found, treat image as single sprite
- [ ] Report detected grid dimensions to stderr

### 6.4 Palette Extraction

- [ ] Collect all unique RGBA values across all cells
- [ ] Skip fully transparent pixels (alpha = 0)
- [ ] Sort by frequency (most common first)
- [ ] Assign names: `$colour-0`, `$colour-1`, etc.
- [ ] Build reverse lookup: `HashMap<Rgba, String>`
- [ ] Write `.palette.md` with frontmatter and `$name: #hex` lines
- [ ] Refactor shared logic with existing `px palette` command

### 6.5 Shape Generation (Pixel-Level)

- [ ] For each cell, build a grid of glyph characters (one per pixel)
- [ ] Assign glyphs to colours by frequency (most common → first available character)
- [ ] Reserve builtin glyphs (`+`, `-`, `|`, `#`, `.`, `x`, ` `)
- [ ] Map transparent pixels to `x`
- [ ] Generate legend: glyph → palette colour reference
- [ ] Write `.shape.md` with frontmatter, `px` code fence, and legend
- [ ] Verify round-trip: `px build` on output should produce identical PNG

### 6.6 Stamp Detection (Structural)

- [ ] Determine candidate block sizes: all (w, h) pairs that evenly divide cell dimensions
- [ ] For each candidate size, extract all blocks from all cells
- [ ] Compute structural hash: replace colours with first-appearance index
- [ ] Group blocks by structural hash
- [ ] Score each candidate size: count of blocks replaced by reuse (higher = better)
- [ ] Pick the best scoring size (or fall back to pixel-level if no reuse found)

### 6.7 Stamp Detection (Colour Variants)

- [ ] For each structural group with 2+ members, identify colour variants
- [ ] Map structural indices to semantic tokens (`$`, `.`, or positional `A`, `B`)
- [ ] Generate `.stamp.md` definitions with semantic/positional pixel grids
- [ ] Assign glyph characters to stamps
- [ ] Generate colour binding tables for each variant
- [ ] Update shape generation to use stamp-level grids and stamp legends

### 6.8 Round-Trip Verification

- [ ] Build generated files with `px build`
- [ ] Compare output PNG against original input pixel-by-pixel
- [ ] Report mismatches with location and expected vs actual colour
- [ ] Add `--verify` flag to `px slice` to run this automatically
- [ ] Verify round-trip for pixel-level shapes (no `--stamps`)
- [ ] Verify round-trip for stamp-detected shapes (`--stamps`)
- [ ] Integration tests: slice → build → diff for each test fixture
- [ ] Test fixtures: single sprite, uniform grid sheet, auto-detected grid, sheet with stamps

**Deliverable**: `px slice tileset.png --cell 16x16 --stamps` outputs `.palette.md`, `.stamp.md`, and `.shape.md` files that round-trip back to the original image.

---

## Future Considerations (Not Planned)

- Animation support (frame sequences in shapes)
- Tilemap autotiling rules
- Code generation (Rust structs, C headers)
- Plugin system
- GUI editor
- Asset browser web app

---

## Milestone Summary

| Phase | Focus | Key Deliverable |
|-------|-------|-----------------|
| 1 | Foundation | Single shape → PNG |
| 2 | Composition | Sheets + metadata |
| 3 | Targets | PICO-8 + Aseprite |
| 4 | Iteration | Watch + preview |
| 5 | Polish | Production ready |
| 6 | Slice | PNG → definition files |

---

## Risk Areas

### Colour Expression Complexity

The full colour expression system (darken, mix, shift, etc.) adds parsing complexity. Mitigation: implement in stages:
1. Hex and references first
2. Single-argument functions (darken, lighten)
3. Two-argument functions (mix)

### Variable Stamp Sizes

Layout calculation with variable-size stamps is non-trivial. Mitigation:
1. Implement fixed grid_size first
2. Add variable sizing later with explicit layout algorithm

### PICO-8 Palette Matching

Reducing arbitrary colours to PICO-8's 16-colour palette requires good quantization. Mitigation:
1. Support user-specified palette mapping
2. Implement multiple quantization algorithms
3. Preview quantization results before export

### Aseprite Format

The .aseprite format is complex (compressed, layered). Mitigation:
1. Use asefile crate initially
2. Fall back to direct implementation if needed
3. Consider Aseprite CLI export as alternative
