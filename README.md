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

**Phase 1.3 complete** - Palette loader with colour expressions.

- `Colour` type with hex parsing (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
- `Palette` type with named colours
- Colour reference resolution with cycle detection
- Variant support (`@variant-name:` blocks)
- Palette inheritance (`inherits:`)
- Builtin default palette ($black, $white, $edge, $fill)
- Colour expressions: `darken`, `lighten`, `saturate`, `desaturate`, `mix`, `alpha`

```markdown
$gold: #F7AD45
$dark-gold: darken($gold, 20%)
$highlight: lighten($gold, 30%)
$muted: desaturate($gold, 50%)
$blend: mix($gold, #FF0000, 30%)
$transparent: alpha($gold, 50%)
```

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
