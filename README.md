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

**Phase 1.7 complete** - Shape renderer.

- Palettes with colour expressions (`darken`, `lighten`, `mix`, etc.)
- Stamps with semantic pixel tokens (`$` edge, `.` fill, `x` transparent)
- Brushes with positional colour tokens (`A`, `B`, `C`) for tiling patterns
- Shaders for palette binding and post-processing effects
- Shapes with ASCII grids and legend-based glyph resolution

```markdown
# player.shape.md
---
name: player-stand
tags: "#player"
---

\`\`\`px
 oo
 ||
+  +
|  |
\`\`\`

---
o: head-fill
```

Rendering resolves glyphs through: legend → stamp glyph → builtins → magenta fallback.

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
