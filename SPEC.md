
# px sprite and map pipeline generation (specification)

#px01 #px #specification #draft

## DSL Specification (v0.1)

### File Types

| Extension     | Purpose                          | Body Format                  |
| ------------- | -------------------------------- | ---------------------------- |
| `.palette.md` | Named colours and variants       | `$name: #hex` lines          |
| `.brush.md`   | Tiling patterns                  | Pixel grid with A/B tokens   |
| `.stamp.md`   | Pixel art with default glyph     | Pixel grid with $/./x tokens |
| `.shader.md`  | Palette + lighting + effects     | Key-value settings           |
| `.shape.md`   | Drawable ASCII compositions      | ASCII art + optional legend  |
| `.prefab.md`  | Shape compositions               | ASCII grid + legend          |
| `.map.md`     | Level layouts                    | ASCII grid + legend          |
| `.target.md`  | Output configuration             | Key-value settings           |

### Common Structure

Each definition follows this structure:

````markdown
---
yaml frontmatter (name, tags, etc.)
---

```px
body content (pixel/layout grids)
```
````

The `px` code fence renders properly in markdown viewers (Hugo, Obsidian) and enables syntax highlighting.

#### Multiple Definitions

Files can contain multiple definitions. Each definition starts with a YAML frontmatter block containing at least `name:`:

````markdown
---
name: first-item
---

```px
...
```

---
name: second-item
---

```px
...
```
````

#### Legend Footer

Shapes, prefabs, and maps can include a legend section after the code block:

````markdown
---
name: level-1
tags: #level
---

```px
##T##
#   #
#####
```

---
#: wall
T: tower
" ": empty
````

The legend maps single characters to stamps, brushes, shapes, or prefabs. Use quotes for special characters like space.

**Legend scoping in multi-definition files:**

Each legend applies to the definition immediately above it. A new `---` with `name:` starts a fresh definition:

````markdown
---
name: shape-a
---

```px
AB
```

---
A: stamp-one
B: stamp-two

---
name: shape-b
---

```px
XY
```

---
X: stamp-three
Y: stamp-four
````

---

### Palette

```yaml
---
name: dungeon
---
$dark: #1a1a2e
$mid: #2d2d44
$light: #4a4a68
$edge: $dark
$fill: $mid

@light-mode:
  $dark: #4a4a68
  $mid: #6e6e8a
```

**Rules:**

- `$name` defines a color
- Can reference other colors: `$edge: $dark`
- `@variant` blocks override colors when `--variant=` is passed
- Inheritance: `inherits: other-palette`

---

### Brush

Brushes are pixel patterns with positional colour tokens. Unlike stamps (which use semantic tokens like `$` and `.`), brushes use letters (`A`, `B`, `C`) that are bound to colours at usage time.

````markdown
---
name: checker
---

```px
AB
BA
```
````

**Rules:**

- Body defines a pixel pattern grid
- Letters (`A`, `B`, etc.) are bound to palette colours at usage
- Builtins: `solid`, `checker`, `diagonal-l`, `diagonal-r`, `h-line`, `v-line`, `noise`

**Brush vs Stamp:**

| | Brush | Stamp |
|---|-------|-------|
| Tokens | Positional (`A`, `B`, `C`) | Semantic (`$`, `.`, `x`) |
| Colour binding | At usage time | Via palette/shader |
| Default glyph | No | Yes (`glyph: B`) |

**Application modes:**

Both brushes and stamps can be placed once (stamp mode) or tiled (fill mode):

```yaml
# Stamps - semantic colours from palette
B: brick                                    # single placement (shorthand)
B: { stamp: brick }                         # single placement (explicit)
~: { fill: brick }                          # tiled fill

# Brushes - colours bound at usage
C: { stamp: checker, A: $edge, B: $fill }  # single placement
~: { fill: checker, A: $edge, B: $fill }   # tiled fill
```

Unbound tokens default to transparent.

---

### Stamp

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

- `glyph` is the character typed in shapes/prefabs/maps
- Body defines pixels; dimensions are the stamp size
- `$` = edge colour, `.` = fill colour, `x` = transparent
- Stamps can be any size; padded/clipped to grid_size if set in brush

---

### Shader

Shaders define palette binding, lighting, and layer effects applied at render time.

```yaml
---
name: dungeon-dark
palette: dungeon
palette_variant: dark-mode
---
lighting: ambient
ambient_color: $dark
effects:
  - type: vignette
    strength: 0.3
  - type: scanlines
    opacity: 0.1
```

**Rules:**

- `palette`: Required; which palette to use for colour resolution
- `palette_variant`: Optional; activates a `@variant` block from the palette
- `lighting`: Optional; lighting model (`ambient`, `directional`, etc.)
- `effects`: Optional; list of post-processing effects
- Inheritance: `inherits: other-shader`

**Shader resolution:**

1. CLI flag `--shader=name` overrides all
2. Maps/prefabs can set `shader: name`
3. Falls back to `default` shader (just uses `default` palette)

---

### Shape

Shapes are ASCII compositions that reference stamps via glyph characters.

````markdown
---
name: wall-segment
tags: #wall #solid
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
- **Legend** (after `---`) defines local glyph mappings
- Stamps declare default glyphs; legend can override or add mappings
- `tags` for metadata export (hashtag prefix is optional, stripped in output)
- Legend syntax supports both placement modes:
  - Single: `B: brick` or `B: { stamp: brick }`
  - Tiled: `~: { fill: checker, A: $edge, B: $fill }`

**Glyph resolution order:**

1. Shape's legend (local overrides)
2. Stamp's declared glyph (`glyph: B` in stamp file)
3. Builtin defaults (`+`, `-`, `|`, `#`, `.`, `x`)

---

### Prefab

````markdown
---
name: tower
tags: #structure
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

**Rules:**

- Body is ASCII grid of placement
- Legend after second `---` maps characters to shape/prefab names
- Whitespace is literal positioning
- Can reference other prefabs (nested)

---

### Map

````markdown
---
name: level-1
tags: #level #dungeon
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

**Rules:**

- Same structure as prefab
- Semantically distinct (level vs reusable component)
- `empty` is a reserved name (transparent/no output)

---

### Target

```yaml
---
name: pico8
format: p8
---
sheet: 128x128
tile: 8x8
colors: 16
palette_mode: indexed
```

```yaml
---
name: web
format: png
---
scale: 4
sheet: auto
```

**Rules:**

- `format`: output type (`png`, `p8`, `spritesheet`, etc.)
- `sheet`: sprite sheet dimensions or `auto`
- `scale`: integer upscale for crisp pixels
- `palette_mode`: `indexed` (constrained) or `rgba` (full color)

---

## Builtin Defaults

### Builtin Stamps

|Glyph|Name|Size|Description|
|---|---|---|---|
|`+`|`corner`|1×1|Single edge-color pixel|
|`-`|`edge-h`|1×1|Single edge-color pixel|
|`\|`|`edge-v`|1×1|Single edge-color pixel|
|`#`|`solid`|1×1|Single edge-color pixel|
|`.`|`fill`|1×1|Single fill-color pixel|
|`x`|`transparent`|1×1|Transparent pixel|
||`space`|1×1|Fill-color pixel (default)|

### Builtin Brushes (Patterns)

|Name|Pattern|Description|
|---|---|---|
|`solid`|—|Single colour fill|
|`checker`|`AB`/`BA`|2×2 checkerboard|
|`diagonal-r`|`AB`/`BA` offset|Diagonal lines (/)|
|`diagonal-l`|`BA`/`AB` offset|Diagonal lines (\)|
|`h-line`|`A`/`B`|Horizontal stripes|
|`v-line`|`AB`|Vertical stripes|

### Builtin Palette

```yaml
---
name: default
---
$black: #000000
$white: #ffffff
$edge: $black
$fill: $white
```

### Builtin Shader

```yaml
---
name: default
palette: default
---
# No effects, just palette binding
```

---

## Metadata Output

Basic JSON alongside rendered output:

```json
{
  "name": "level-1",
  "size": [256, 256],
  "grid": [32, 32],
  "shapes": [
    {
      "name": "wall-segment",
      "tags": ["wall", "solid"],
      "positions": [[0,0], [8,0], [16,0], ...]
    },
    {
      "name": "tower",
      "tags": ["structure"],
      "positions": [[16, 16], [48, 16]]
    }
  ]
}
```

Enough for a game to build collision maps or tile lookups. Richer codegen (Rust structs, C headers, etc.) is a future feature.

---

## Implementation Order

**Phase 1: Core Pipeline**

1. File parser (YAML frontmatter + body extraction)
2. Palette loading and colour resolution
3. Stamp loading and pixel grid parsing
4. Brush loading (pattern grids)
5. Shape rendering (ASCII → pixels via glyph resolution)
6. PNG output (single shape)

**Phase 2: Composition**

1. Prefab loading and shape placement
2. Map loading (same as prefab, larger)
3. Sprite sheet packing
4. JSON metadata export

**Phase 3: Shaders & Targets**

1. Shader loading and palette binding
2. Palette variants (`@light-mode`)
3. Target profiles (PICO-8 format, scaling, etc.)
4. Build command with `--target`, `--shader`, `--brush` flags

**Phase 4: Polish**

1. Watch mode for iteration
2. Preview server (HTML output)
3. Validation and warnings (stamp size mismatches, missing refs)
4. Multi-file shape/stamp definitions

---

## Technical Considerations

**Rust crates:**

- `image` — PNG/etc. output
- `serde` + `serde_yaml` — frontmatter parsing
- `walkdir` — file discovery
- `clap` — CLI

**Key data structures:**

```rust
struct Palette {
    colours: HashMap<String, Rgba>,
    variants: HashMap<String, HashMap<String, Rgba>>,
}

struct Stamp {
    name: String,
    glyph: Option<char>,              // Default glyph (self-declared)
    pixels: Vec<Vec<PixelType>>,      // Edge, Fill, Transparent
}

struct Brush {
    name: String,
    pixels: Vec<Vec<char>>,           // Pattern grid (A, B, C tokens)
}

struct Shader {
    palette: String,
    variant: Option<String>,
    effects: Vec<Effect>,
}

struct Shape {
    name: String,
    tags: Vec<String>,
    grid: Vec<Vec<char>>,
    legend: HashMap<char, LegendEntry>,  // Local glyph overrides
}

enum LegendEntry {
    StampRef(String),                           // B: brick
    BrushRef { name: String, colors: HashMap<char, String> },  // { stamp: checker, A: $edge }
    Fill { name: String, colors: HashMap<char, String> },      // { fill: checker, A: $edge }
}
```

**Glyph resolution order:**

1. Shape's legend (local overrides)
2. Stamp's declared glyph (`glyph: B` in stamp file)
3. Builtin defaults (`+`, `-`, `|`, `#`, `.`, `x`, ` `)

**Rendering pipeline:**

```
Shape.grid + Legend + Stamps → Vec<Vec<Pixels>> + Shader → Vec<Vec<Rgba>> → Image
```

1. Resolve glyphs to stamps/brushes via legend and stamp defaults
2. Expand stamps to pixel tokens (or tile brushes for fills)
3. Shader applies palette colours and effects
