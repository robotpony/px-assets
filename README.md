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

**Phase 2.8 complete** - Directory-aware build and `px init`.

`px build` now discovers assets automatically:

```bash
# Build everything in current directory (reads px.yaml if present)
px build

# Build from a project directory
px build examples/mission-improbable/
px build examples/pac-man/ --scale 4

# Build specific files (still works)
px build shapes/*.shape.md prefabs/*.prefab.md -o dist
```

`px init` generates a `px.yaml` manifest from discovered assets:

```bash
px init              # Scan current directory
px init my-project/  # Scan a specific directory
px init --force      # Overwrite existing px.yaml
```

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

Each character in the grid maps to a shape (or another prefab) by name. Nested prefabs are resolved automatically via topological sort. Building a prefab outputs both a PNG and a JSON file with instance metadata:

```json
{
  "name": "tower",
  "size": [4, 16],
  "tags": [],
  "grid": [1, 4],
  "cell_size": [4, 4],
  "shapes": [
    { "name": "roof", "positions": [[0, 0]] },
    { "name": "wall-segment", "positions": [[0, 4], [0, 8]] },
    { "name": "door", "positions": [[0, 12]] }
  ]
}
```

Individual shapes also export JSON metadata (`name`, `size`, `tags`) alongside their PNGs.

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
- Sprite sheet packing with `--sheet` (shelf algorithm, TexturePacker-compatible JSON)
- Target profiles for bundling output settings (`--target=web`, `--target=sheet`, or custom `.target.md` files)
- PNG output with integer scaling

```bash
# Build everything in current directory
px build

# Build from a project directory
px build my-project/

# Build a shape file to PNG
px build shapes/wall.shape.md -o dist --scale 4

# Build with custom shader
px build shapes/*.shape.md --shader=dungeon -o dist

# Pack all sprites into a single sheet
px build shapes/*.shape.md prefabs/*.prefab.md --sheet -o dist

# Sheet with 2px padding between sprites
px build shapes/*.shape.md --sheet --padding 2 -o dist

# Build with a named target profile
px build shapes/*.shape.md --target=web -o dist

# Target with CLI overrides (CLI scale wins over target scale)
px build shapes/*.shape.md --target=web --scale 4 -o dist

# Build with a custom target file
px build shapes/*.shape.md --target=retro.target.md -o dist
```

Targets bundle output settings into named profiles. Two builtins are provided: `web` (individual PNGs, all defaults) and `sheet` (auto sheet packing). Custom targets use `.target.md` files:

````markdown
# retro.target.md
---
name: retro
format: png
---

scale: 4
sheet: auto
padding: 2
shader: dark
````

Settings merge in priority order: CLI flags > target profile > per-asset frontmatter > defaults.

Shapes can also specify scale in frontmatter:

```markdown
---
name: player-stand
scale: 2
---
```

CLI `--scale` overrides frontmatter when specified.

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
