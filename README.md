# px

A CLI tool for generating sprites and sprite maps from text-based definitions.

## What it does

Define sprites using ASCII art in markdown files. Apply colour palettes and styles. Export to PNG, PICO-8, or Aseprite formats.

```
# wall.shape.md
---
name: wall-segment
---
+--+
|..|
|..|
+--+
```

The same shape can be rendered with different palettes, at different scales, for different target platforms.

## Status

Early development. See [SPEC.md](SPEC.md) for the planned DSL.
