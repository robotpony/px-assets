# px - Sprite and Map Pipeline Generator

A Rust CLI tool that transforms markdown-style definition files into sprites and sprite maps for various platforms.

## Project Overview

`px` uses a domain-specific language in markdown files to define:
- **Palettes** (`.palette.md`) - Named colours and variants
- **Brushes** (`.brush.md`) - Fill patterns
- **Stamps** (`.stamp.md`) - Character-to-pixel mappings
- **Shaders** (`.shader.md`) - Palette binding + effects
- **Shapes** (`.shape.md`) - Drawable ASCII compositions
- **Prefabs** (`.prefab.md`) - Shape compositions
- **Maps** (`.map.md`) - Level layouts
- **Targets** (`.target.md`) - Output configuration

## Tech Stack

- **Language**: Rust
- **Key crates**: `image`, `serde`, `serde_yaml`, `walkdir`, `clap`

## Architecture

Rendering pipeline:
```
Shape.grid + Style → Vec<Vec<Stamp>> → Vec<Vec<Rgba>> → Image
```

Each cell in the grid maps to a stamp, stamps expand to pixels, pixels get final colours from palette.

## Commands

```bash
cargo build              # Build the project
cargo test               # Run tests
cargo run -- [args]      # Run with arguments
```

## Code Style

- Prefer explicit error handling with `Result` types
- Use descriptive variable names matching the domain (stamp, glyph, palette, etc.)
- Keep parsing separate from rendering logic
- Write tests for each file type parser

See SPEC.md for the DSL specification and PLAN.md for implementation status.
