# px Libraries

Crate selection with rationale.

## Core Dependencies

### Parsing & Serialization

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.x | Serialization framework |
| `serde_yaml` | 0.9.x | YAML frontmatter parsing |
| `serde_json` | 1.x | JSON metadata output |

**Rationale**: serde is the standard. YAML for human-friendly frontmatter, JSON for machine-readable output.

### Image Processing

| Crate | Version | Purpose |
|-------|---------|---------|
| `image` | 0.25.x | PNG reading/writing, pixel manipulation |
| `palette` | 0.7.x | Colour space conversions (RGB, HSL, etc.) |

**Rationale**:
- `image` is the standard Rust image library. Handles PNG, supports RGBA.
- `palette` for proper colour math (darken/lighten/shift operate in perceptually uniform space).

### CLI

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.x | Argument parsing with derive macros |
| `clap_complete` | 4.x | Shell completion generation |

**Rationale**: clap is mature, well-documented, and supports both derive and builder patterns.

### File System

| Crate | Version | Purpose |
|-------|---------|---------|
| `walkdir` | 2.x | Recursive directory traversal |
| `notify` | 6.x | File system watching |
| `globset` | 0.4.x | Glob pattern matching |

**Rationale**: Standard choices. notify is cross-platform and handles debouncing.

### Error Handling

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 1.x | Derive Error implementations |
| `miette` | 7.x | Diagnostic error reporting |

**Rationale**:
- `thiserror` for clean error types.
- `miette` for pretty error messages with source spans (like the rustc-style errors in DESIGN.md).

### Concurrency

| Crate | Version | Purpose |
|-------|---------|---------|
| `rayon` | 1.x | Parallel iteration |

**Rationale**: Simple API, work-stealing scheduler, well-suited for batch processing.

### Caching

| Crate | Version | Purpose |
|-------|---------|---------|
| `seahash` | 4.x | Fast content hashing |
| `bincode` | 1.x | Binary serialization for cache files |

**Rationale**: seahash is fast for non-cryptographic hashing. bincode for compact cache storage.

## Output Format Support

### PICO-8

| Crate | Version | Purpose |
|-------|---------|---------|
| `color_quant` | 1.x | Colour quantization |

**Rationale**: Needed to reduce colours to 16-colour indexed palette. Implements median-cut and other algorithms.

Custom code required for:
- P8 file format writing (text-based, straightforward)
- Ordered dithering (standard Bayer matrix implementation)
- Floyd-Steinberg dithering

### Aseprite

| Crate | Version | Purpose |
|-------|---------|---------|
| `asefile` | 0.3.x | Aseprite file format |

**Rationale**: Third-party crate for reading/writing .aseprite format. Handles layers, frames, palette.

**Note**: If `asefile` proves insufficient, the Aseprite format is documented and can be implemented directly using `binrw` or similar.

## Preview Server

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.7.x | HTTP server framework |
| `tokio` | 1.x | Async runtime |
| `tower-http` | 0.5.x | Static file serving, CORS |
| `tokio-tungstenite` | 0.21.x | WebSocket for hot reload |

**Rationale**: axum is lightweight, fast, and ergonomic. WebSocket for push-based hot reload.

## Development Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `insta` | 1.x | Snapshot testing |
| `pretty_assertions` | 1.x | Diff-based test failures |
| `tempfile` | 3.x | Temporary directories for tests |
| `criterion` | 0.5.x | Benchmarking |

**Rationale**: Snapshot testing is ideal for pixel output verification. criterion for performance regression testing.

## Dependency Summary

```toml
[dependencies]
# Parsing
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"

# Image
image = "0.25"
palette = "0.7"

# CLI
clap = { version = "4", features = ["derive"] }
clap_complete = "4"

# Filesystem
walkdir = "2"
notify = "6"
globset = "0.4"

# Error handling
thiserror = "1"
miette = { version = "7", features = ["fancy"] }

# Concurrency
rayon = "1"

# Cache
seahash = "4"
bincode = "1"

# Output formats
color_quant = "1"
asefile = "0.3"

# Preview server
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["fs", "cors"] }
tokio-tungstenite = "0.21"

[dev-dependencies]
insta = "1"
pretty_assertions = "1"
tempfile = "3"
criterion = "0.5"
```

## Alternatives Considered

### Image Processing

**Considered**: `imageproc` for additional image operations.
**Decision**: Not needed initially. Standard `image` crate sufficient for pixel-level operations.

### CLI

**Considered**: `argh` (simpler, smaller binary).
**Decision**: clap's ecosystem (completions, better help) worth the size.

### Error Reporting

**Considered**: `ariadne` (similar to miette, different API).
**Decision**: miette integrates better with thiserror.

### YAML

**Considered**: `yaml-rust` (pure Rust, no serde).
**Decision**: serde_yaml for consistency and typed deserialization.

### Preview Server

**Considered**: `actix-web`, `warp`.
**Decision**: axum for simplicity and tower ecosystem integration.

### Hashing

**Considered**: `xxhash-rust`, `blake3`.
**Decision**: seahash is fast and simple. blake3 overkill for caching.

## Notes on Stability

- All crates at 1.x or well-maintained 0.x versions
- No nightly Rust features required
- MSRV target: 1.75+ (for async traits, impl Trait in return position)
