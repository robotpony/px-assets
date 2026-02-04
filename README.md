# px

A CLI tool for generating sprites and sprite maps from text-based definitions.

## What it does

Define sprites using ASCII art in markdown files. Apply colour palettes and styles. Export to PNG, PICO-8, or Aseprite formats.

````markdown
# wall.shape.md
---
name: wall-segment
---

```px
+--+
|..|
|..|
+--+
```
````

The same shape can be rendered with different palettes, at different scales, for different target platforms.

## Status

**Phase 1.8 complete** - PNG output.

- Palettes with colour expressions (`darken`, `lighten`, `mix`, etc.)
- Stamps with semantic pixel tokens (`$` edge, `.` fill, `x` transparent)
- Brushes with positional colour tokens (`A`, `B`, `C`) for tiling patterns
- Shaders for palette binding and post-processing effects
- Shapes with ASCII grids and legend-based glyph resolution
- PNG output with integer scaling

```bash
# Build a shape file to PNG
px build shapes/wall.shape.md -o dist --scale 4

# Build with custom shader
px build shapes/*.shape.md --shader=dungeon -o dist
```

Shapes can also specify scale in frontmatter:

```markdown
---
name: player-stand
scale: 2
---
```

CLI `--scale` overrides frontmatter when specified.

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
