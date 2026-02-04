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

**Phase 1.4 complete** - Stamp loader.

- Palettes with colour expressions (`darken`, `lighten`, `mix`, etc.)
- Stamps with semantic pixel tokens (`$` edge, `.` fill, `x` transparent)
- 7 builtin stamps: `corner`, `edge-h`, `edge-v`, `solid`, `fill`, `transparent`, `space`

````markdown
# brick.stamp.md
---
name: brick
glyph: B
---

```px
$$$$$$$$
$......$
$......$
$$$$$$$$
```
````

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
