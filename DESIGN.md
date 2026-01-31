# px Design

User-facing interfaces: CLI, file formats, and preview system.

## Command Line Interface

### Command Structure

```
px <command> [options] [targets...]
px <target-name>              # Shorthand for: px build --target=<target-name>
```

### Commands

#### `px build`

Build assets for one or more targets.

```bash
px build                      # Build default target
px build --target=web         # Build specific target
px build --target=web,pico8   # Build multiple targets
px build web pico8            # Shorthand for above

px build --style=dark         # Override default style
px build --variant=light-mode # Apply palette variant
px build --output=dist/       # Override output directory

px build --clean              # Clean output before build
px build --no-cache           # Ignore cache, rebuild all
px build --verbose            # Show detailed progress
```

#### `px watch`

Watch for changes and rebuild.

```bash
px watch                      # Watch, build default target
px watch web                  # Watch, build specific target
px watch --preview            # Also start preview server

px watch --debounce=200       # Debounce interval in ms (default: 100)
```

#### `px preview`

Start live preview server.

```bash
px preview                    # Start server on default port
px preview --port=8080        # Specify port
px preview --open             # Open browser automatically

# Preview shows:
# - Rendered sprites/shapes
# - Sprite sheet with grid overlay
# - Click-to-inspect with metadata
# - Hot reload on file changes
```

#### `px validate`

Check project for errors without building.

```bash
px validate                   # Validate all files
px validate shapes/           # Validate specific directory
px validate hero.shape.md     # Validate specific file

px validate --strict          # Treat warnings as errors
```

#### `px init`

Initialize a new px project.

```bash
px init                       # Create px.yaml and example files
px init --minimal             # Just px.yaml, no examples
```

#### `px list`

List discovered assets.

```bash
px list                       # List all assets by type
px list shapes                # List only shapes
px list --deps                # Show dependency tree
px list --unused              # Show unreferenced assets
```

### Target Shorthand

Any defined target can be invoked directly:

```bash
px web                        # Equivalent to: px build --target=web
px pico8                      # Equivalent to: px build --target=pico8
px pico8 web                  # Build both targets
```

### Global Options

```bash
px --version                  # Show version
px --help                     # Show help
px -C <dir>                   # Change working directory
px --config=px.yaml           # Specify config file
```

## File Formats

### Common Structure

All definition files share:

```markdown
---
yaml frontmatter
---
body content

---
optional legend (prefab/map only)
```

Multiple definitions per file, separated by `---` with `name:` in each frontmatter.

### Palette (`.palette.md`)

```yaml
---
name: dungeon
inherits: base          # Optional inheritance
---
# Basic colours
$dark: #1a1a2e
$mid: #2d2d44
$light: #4a4a68

# References
$edge: $dark
$fill: $mid

# Expressions
$shadow: darken($fill, 20%)
$highlight: lighten($edge, 15%)
$accent: shift($mid, 180)          # Hue rotation
$muted: desaturate($light, 30%)
$blend: mix($dark, $light, 40%)
$ghost: alpha($fill, 0.5)

# Variants (activated with --variant=)
@light-mode:
  $dark: #4a4a68
  $mid: #6e6e8a
  $light: #9090a8

@high-contrast:
  $dark: #000000
  $light: #ffffff
```

**Colour expression functions:**

| Function | Syntax | Description |
|----------|--------|-------------|
| `darken` | `darken($color, 20%)` | Reduce lightness |
| `lighten` | `lighten($color, 20%)` | Increase lightness |
| `saturate` | `saturate($color, 20%)` | Increase saturation |
| `desaturate` | `desaturate($color, 20%)` | Reduce saturation |
| `shift` | `shift($color, 180)` | Rotate hue (degrees) |
| `mix` | `mix($a, $b, 50%)` | Blend two colours |
| `alpha` | `alpha($color, 0.5)` | Set opacity |

### Brush (`.brush.md`)

```yaml
---
name: checker
---
AB
BA
```

Pattern tiles infinitely. Letters bind to colours via style.

**Builtins:** `solid`, `checker`, `diagonal-l`, `diagonal-r`, `h-line`, `v-line`, `noise`

### Stamp (`.stamp.md`)

```yaml
---
name: brick
glyph: B
---
$$$$$$$$
$......$
$......$
$$$$$$$$
```

**Pixel tokens:**

| Token | Meaning |
|-------|---------|
| `$` | Edge colour (`$edge` from palette) |
| `.` | Fill colour (`$fill` from palette) |
| `x` | Transparent |
| `A-Z` | Brush pattern colours |
| `0-9` | Direct palette index (indexed modes) |

### Style (`.style.md`)

```yaml
---
name: brick-red
palette: dungeon              # Required
palette_variant: light-mode   # Optional
inherits: base-style          # Optional
grid_size: 8x8                # Optional (auto = variable stamps)
---
# Glyph to stamp mappings
+: corner
-: edge-h
|: edge-v
B: brick

# Inline definitions
" ": { brush: solid, color: $fill }
x: transparent

# Brush with pattern
~: { brush: checker, colors: [$dark, $mid] }
```

**Grid size options:**

- `8x8`, `16x16`, etc.: Fixed grid; stamps pad/clip to fit
- `auto`: Variable stamp sizes; layout calculated dynamically

### Shape (`.shape.md`)

```yaml
---
name: wall-segment
tags: [wall, solid, collidable]
---
+--+
|BB|
|BB|
+--+
```

Shapes have no embedded style. Style applied at render time.

### Prefab (`.prefab.md`)

```yaml
---
name: tower
tags: [structure, tall]
---
C
W
W
W
B

---
C: tower-cap
W: wall-segment
B: tower-base
```

Grid positions shapes/prefabs. Whitespace is literal. Can nest prefabs.

### Map (`.map.md`)

```yaml
---
name: level-1
tags: [level, dungeon, tutorial]
style: dungeon-dark           # Optional style override
---
################
#              #
#  T        T  #
#==+========+==#
################

---
#: wall-segment
=: platform
T: tower
+: pillar-base
" ": empty
```

Semantically a level; structurally identical to prefab. `empty` is reserved (no output).

### Target (`.target.md`)

```yaml
---
name: web
format: png
---
scale: 4
sheet: auto                   # or 256x256, 512x512, etc.
metadata: true                # Export JSON alongside
padding: 1                    # Pixels between sprites
```

```yaml
---
name: pico8
format: p8
---
sheet: 128x128
tile: 8x8
colors: 16
palette_mode: indexed
dither: ordered               # none, ordered, floyd-steinberg
quantize: median-cut          # Algorithm for colour reduction
```

```yaml
---
name: aseprite
format: aseprite
---
layers: true                  # Preserve shape layers
frames: false                 # Single frame per shape
```

**Format-specific options:**

| Format | Options |
|--------|---------|
| `png` | `scale`, `sheet`, `padding`, `metadata` |
| `p8` | `sheet`, `tile`, `colors`, `palette_mode`, `dither`, `quantize` |
| `aseprite` | `layers`, `frames`, `palette` |

## Output Structure

### Default Layout

```
dist/
├── web/
│   ├── sprites.png
│   ├── sprites.json
│   ├── shapes/
│   │   ├── wall-segment.png
│   │   └── tower.png
│   └── maps/
│       ├── level-1.png
│       └── level-1.json
├── pico8/
│   └── sprites.p8
└── aseprite/
    └── sprites.aseprite
```

### Metadata JSON

```json
{
  "name": "sprites",
  "size": [256, 256],
  "shapes": [
    {
      "name": "wall-segment",
      "tags": ["wall", "solid"],
      "frame": { "x": 0, "y": 0, "w": 32, "h": 32 },
      "origin": [16, 32]
    },
    {
      "name": "tower",
      "tags": ["structure"],
      "frame": { "x": 32, "y": 0, "w": 32, "h": 128 },
      "origin": [16, 128]
    }
  ]
}
```

### Map Metadata

```json
{
  "name": "level-1",
  "size": [512, 160],
  "grid": { "cell": [32, 32], "cols": 16, "rows": 5 },
  "instances": [
    {
      "shape": "wall-segment",
      "tags": ["wall", "solid"],
      "cell": [0, 0],
      "position": [0, 0]
    },
    {
      "shape": "tower",
      "tags": ["structure"],
      "cell": [3, 2],
      "position": [96, 64]
    }
  ]
}
```

## Preview Server

### Interface

```
┌─────────────────────────────────────────────────────────────┐
│  px preview                                    localhost:3000│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │
│ │ Shapes  │ │ Prefabs │ │  Maps   │ │ Sheets  │            │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐                   │
│   │      │  │      │  │      │  │      │                   │
│   │ wall │  │brick │  │tower │  │ cap  │                   │
│   │      │  │      │  │      │  │      │                   │
│   └──────┘  └──────┘  └──────┘  └──────┘                   │
│                                                             │
│   Click to inspect                                          │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Inspector: tower                                            │
│ Tags: structure, tall                                       │
│ Size: 32x128                                                │
│ Stamps: corner, edge-h, edge-v, brick                       │
│ Source: prefabs/tower.prefab.md                             │
└─────────────────────────────────────────────────────────────┘
```

### Features

- **Hot reload**: Changes rebuild and update browser automatically
- **Zoom**: Scroll to zoom, preserving pixel crispness
- **Grid overlay**: Toggle to see stamp boundaries
- **Inspector**: Click any sprite for metadata
- **Palette view**: See all palette colours with names
- **Compare variants**: Side-by-side palette variant comparison

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1-4` | Switch tabs (Shapes, Prefabs, Maps, Sheets) |
| `g` | Toggle grid overlay |
| `+`/`-` | Zoom in/out |
| `0` | Reset zoom |
| `v` | Cycle variants |
| `Esc` | Close inspector |

## Error Messages

### Format

```
warning: missing stamp 'brick' in style 'dungeon'
  --> shapes/wall.shape.md:4:2
   |
 4 | |BB|
   |  ^ glyph 'B' mapped to 'brick' which doesn't exist
   |
   = using magenta placeholder

error: circular reference in palette
  --> palettes/dungeon.palette.md:8:10
   |
 8 | $a: $b
 9 | $b: $a
   |
   = $a -> $b -> $a

warning: stamp size mismatch
  --> stamps/brick.stamp.md
   |
   = stamp 'brick' is 8x8, style 'retro' expects 16x16
   = stamp will be centered with padding
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Build completed with warnings |
| 2 | Validation errors (--strict mode) |
| 3 | Fatal error (missing files, parse failure) |
