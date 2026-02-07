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

**Phase 1.9 complete** - Validation system.

`px validate` checks assets for missing references, unused legends, mismatched stamp sizes, and more:

```bash
# Validate a project directory
px validate shapes/ prefabs/

# Validate before building
px build --validate shapes/*.shape.md -o dist
```

Maps define level layouts using the same grid+legend format as prefabs, with JSON metadata export:

````markdown
# dungeon-1.map.md
---
name: dungeon-1
---

```px
WWWW
W..W
W..W
WWDW
```

---
W: wall-segment
D: door
.: empty
````

Each character maps to a shape or prefab by name. The reserved name `empty` produces transparent cells with no metadata. Building a map outputs both a PNG and a JSON file with instance positions:

```json
{
  "name": "dungeon-1",
  "size": [32, 32],
  "grid": [4, 4],
  "cell_size": [8, 8],
  "shapes": [
    { "name": "wall-segment", "tags": [], "positions": [[0,0], [8,0], ...] },
    { "name": "door", "tags": [], "positions": [[16,24]] }
  ]
}
```

Prefabs composite pre-rendered shapes into larger images using an ASCII placement grid:

````markdown
# tower.prefab.md
---
name: tower
---

```px
R
W
W
D
```

---
R: roof
W: wall-segment
D: door
````

Each character in the grid maps to a shape (or another prefab) by name. Nested prefabs are resolved automatically via topological sort.

```bash
# Build shapes, prefabs, and maps together
px build shapes/*.shape.md prefabs/tower.prefab.md maps/dungeon-1.map.md -o dist --scale 4
```

Projects can use a `px.yaml` manifest for configuration:

```yaml
# px.yaml
sources:
  - shapes/
  - palettes/
output: dist/sprites
shader: dungeon-dark
scale: 4
excludes:
  - "*.bak"
  - "**/temp/*"
```

Or rely on convention-based discovery (scans current directory for `.shape.md`, `.palette.md`, etc.).

- Palettes with colour expressions (`darken`, `lighten`, `mix`, etc.)
- Stamps with semantic pixel tokens (`$` edge, `.` fill, `x` transparent)
- Brushes with positional colour tokens (`A`, `B`, `C`) for tiling patterns
- Shaders for palette binding and post-processing effects
- Shapes with ASCII grids and legend-based glyph resolution
- Prefabs for compositing shapes into larger images (nested prefab support)
- Maps for level layouts with JSON metadata export (instance positions, grid info)
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
