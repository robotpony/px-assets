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

- [ ] Implement bin packing algorithm
- [ ] Support fixed sheet size
- [ ] Support auto-sizing
- [ ] Add configurable padding
- [ ] Generate frame metadata (x, y, w, h)

### 2.6 Metadata Export

- [ ] Generate shape metadata JSON
- [ ] Generate sheet metadata JSON
- [ ] Generate map metadata JSON (instances, grid info)

### 2.7 Target System

- [ ] Parse target definitions
- [ ] Implement PNG target writer
- [ ] Wire target selection to CLI (`--target=`)

**Deliverable**: `px build --target=web` outputs sprite sheet + JSON.

---

## Phase 3: Additional Targets

**Goal**: PICO-8 and Aseprite output.

### 3.1 PICO-8 Output

- [ ] Implement colour quantization (16 colours)
- [ ] Implement ordered dithering
- [ ] Implement Floyd-Steinberg dithering
- [ ] Generate P8 sprite data format
- [ ] Write .p8 file
- [ ] Map to PICO-8 standard palette

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

- [ ] Implement `px init`
- [ ] Implement `px list`
- [ ] Add shell completions (clap_complete)
- [ ] Add `--verbose` and `--quiet` modes
- [ ] Add colour output (terminal colours)

### 5.2 Advanced Validation

- [ ] Warn on unused assets
- [ ] Warn on shadowed definitions
- [ ] Check for palette colour unused in shapes
- [ ] Suggest fixes for common errors

### 5.3 Documentation

- [ ] Write user guide
- [ ] Document file format spec (from SPEC.md)
- [ ] Add inline examples in error messages
- [ ] Generate man pages

### 5.4 Testing

- [ ] Snapshot tests for rendered output
- [ ] Unit tests for parsers
- [ ] Integration tests for full pipeline
- [ ] Performance benchmarks

**Deliverable**: Production-ready CLI tool.

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
