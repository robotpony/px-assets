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

px build --shader=dark        # Override default shader
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

Each definition follows:

````markdown
---
yaml frontmatter (name, tags, etc.)
---

```px
body content
```
````

**Multiple definitions** per file are supported. Each starts with `---` and a `name:` field:

````markdown
---
name: first
---

```px
...
```

---
name: second
---

```px
...
```
````

**Legend footer** (prefab/map only) follows the code block:

````markdown
---
name: level
---

```px
#T#
```

---
#: wall
T: tower
````

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

Brushes define tiling patterns with positional colour tokens.

````markdown
---
name: checker
---

```px
AB
BA
```

---
name: diagonal-r
---

```px
AB
BA
```

---
name: h-line
---

```px
A
B
```
````

**Rules:**

- Body defines a pixel pattern grid
- Letters (`A`, `B`, etc.) are bound to palette colours at usage time
- Used via legend syntax: `~: { fill: checker, A: $edge, B: $fill }`

**Builtin patterns:** `solid`, `checker`, `diagonal-l`, `diagonal-r`, `h-line`, `v-line`, `noise`

**Brush vs Stamp:**

| | Brush | Stamp |
|---|-------|-------|
| Tokens | Positional (`A`, `B`, `C`) | Semantic (`$`, `.`, `x`) |
| Colour binding | At usage time via legend | Via palette/shader |
| Default glyph | No | Yes (`glyph: B`) |

### Stamp (`.stamp.md`)

Stamps define pixel art with semantic colour tokens and an optional default glyph.

````markdown
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

**Rules:**

- `glyph`: Optional default character for this stamp in shapes
- Body defines pixels using semantic tokens
- Stamps can be any size

**Pixel tokens:**

| Token | Meaning |
|-------|---------|
| `$` | Edge colour (`$edge` from palette) |
| `.` | Fill colour (`$fill` from palette) |
| `x` | Transparent |

**Glyph resolution order:**

1. Shape's legend (local override)
2. Stamp's `glyph:` field (self-declared default)
3. Builtin stamps (`+`, `-`, `|`, `#`, `.`, `x`, ` `)

### Shader (`.shader.md`)

Shaders bind palettes and apply effects at render time.

```yaml
---
name: dungeon-dark
palette: dungeon
palette_variant: dark-mode
inherits: base-shader
---
lighting: ambient
ambient_color: $dark
effects:
  - type: vignette
    strength: 0.3
```

**Rules:**

- `palette`: Required; which palette to use
- `palette_variant`: Optional; activates a `@variant` block
- `effects`: Optional; post-processing effects list
- CLI flag `--shader=name` overrides; maps/prefabs can set `shader: name`

### Shape (`.shape.md`)

````markdown
---
name: wall-segment
tags: #wall #solid #collidable
---

```px
+--+
|BB|
|BB|
+--+
```

---
B: brick
~: { fill: checker, A: $edge, B: $fill }
````

**Rules:**

- Body uses characters that map to stamps or brushes
- Optional legend (after `---`) defines local glyph overrides
- Glyphs resolve via: legend → stamp's `glyph:` → builtins

**Legend syntax:**

```yaml
# Stamps (single placement)
B: brick                                    # shorthand
B: { stamp: brick }                         # explicit

# Brushes placed once (with colour binding)
C: { stamp: checker, A: $edge, B: $fill }

# Tiled fills
~: { fill: brick }                          # stamp tiled
~: { fill: checker, A: $edge, B: $fill }   # brush tiled
```

### Prefab (`.prefab.md`)

````markdown
---
name: tower
tags: #structure #tall
---

```px
C
W
W
W
B
```

---
C: tower-cap
W: wall-segment
B: tower-base
````

Grid positions shapes/prefabs. Whitespace is literal. Can nest prefabs.

### Map (`.map.md`)

````markdown
---
name: level-1
tags: #level #dungeon #tutorial
brush: dungeon
shader: dungeon-dark          # Optional shader override
---

```px
################
#              #
#  T        T  #
#==+========+==#
################
```

---
#: wall-segment
=: platform
T: tower
+: pillar-base
" ": empty
````

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

**Grid sizing:**

The `tile` property (e.g., `tile: 8x8`) controls stamp sizing for targets that need fixed grids. Stamps are padded (centered) or clipped to fit. For variable-size output, omit `tile` or set `tile: auto`.

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
warning: missing stamp 'brick' in brush 'dungeon'
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
   = stamp 'brick' is 8x8, brush 'retro' expects 16x16
   = stamp will be centered with padding
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Build completed with warnings |
| 2 | Validation errors (--strict mode) |
| 3 | Fatal error (missing files, parse failure) |
