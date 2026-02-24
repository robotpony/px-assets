# px

A CLI tool for generating sprites and sprite maps from text-based definitions.

## What it does

Define sprites using ASCII art in markdown files. Apply colour palettes and shaders. Export to PNG sprite sheets with JSON metadata.

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

## Quick start

1. Create a shape file (`hero.shape.md`):

    ````markdown
    ---
    name: hero
    ---

    ```px
    .##.
    ####
    .##.
    .#.#
    ```
    ````

2. Build it:

    ```bash
    px build hero.shape.md -o dist --scale 4
    ```

3. Find `hero.png` and `hero.json` in `dist/`.

## Commands

**`px build`** discovers and renders assets to PNG + JSON.

```bash
px build                              # Build everything in current directory
px build examples/pac-man/ --scale 4  # Build from a project directory
px build shapes/*.shape.md -o dist    # Build specific files
px build --sheet --padding 2 -o dist  # Pack into a sprite sheet
px build --target=web -o dist         # Use a named target profile
```

**`px init`** generates a `px.yaml` manifest from discovered assets.

```bash
px init              # Scan current directory
px init my-project/  # Scan a specific directory
```

**`px palette`** extracts colours from a PNG into `.palette.md` format.

```bash
px palette ref.png --max 16    # 16 most frequent colours
```

**`px validate`** checks assets for missing references, unused legends, and mismatched stamp sizes.

```bash
px validate shapes/ prefabs/
```

## Features

- **Palettes** with colour expressions (`darken`, `lighten`, `mix`, etc.)
- **Stamps** with semantic pixel tokens (`$` edge, `.` fill, `x` transparent)
- **Brushes** with positional colour tokens (`A`, `B`, `C`) for tiling patterns
- **Shaders** for palette binding and post-processing effects
- **Shapes** with ASCII grids and legend-based glyph resolution
- **Prefabs** for compositing shapes into larger images (nested prefab support)
- **Maps** for level layouts with JSON metadata export (instance positions, grid info)
- **Sprite sheet** packing with `--sheet` (shelf algorithm, TexturePacker-compatible JSON)
- **Target profiles** for bundling output settings (`--target=web`, `--target=sheet`, or custom `.target.md` files)
- PNG output with integer scaling

<details>
<summary>Format examples</summary>

### Prefab

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

Each character maps to a shape or another prefab by name. Nested prefabs resolve automatically via topological sort.

### Map

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

The reserved name `empty` produces transparent cells. Building a map outputs both a PNG and a JSON file with instance positions.

### Target

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

### Project manifest

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

</details>

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
