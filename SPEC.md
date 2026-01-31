
# px sprite and map pipeline generation (specificatopm)

#px01 #px #specification #draft

## DSL Specification (v0.1)

### File Types

| Extension     | Purpose                     | Body Format                  |
| ------------- | --------------------------- | ---------------------------- |
| `.palette.md` | Named colors and variants   | `$name: #hex` lines          |
| `.brush.md`   | Fill patterns               | ASCII pattern grid           |
| `.stamp.md`   | Character → pixel mapping   | ASCII pixel grid             |
| `.style.md`   | Binds palette + stamps      | `glyph: stamp-name` mappings |
| `.shape.md`   | Drawable ASCII compositions | ASCII art                    |
| `.prefab.md`  | Shape compositions          | ASCII grid + legend          |
| `.map.md`     | Level layouts               | ASCII grid + legend          |
| `.target.md`  | Output configuration        | Key-value settings           |

### Common Structure

All files follow:

```
---
yaml frontmatter
---
body content

---
optional legend (prefab/map only)
```

Multiple definitions per file allowed, separated by `---` with `name:` in each frontmatter.

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

```yaml
---
name: checker
---
AB
BA
```

**Rules:**

- Body is the tile pattern
- Letters (`A`, `B`, etc.) bind to colors at render time via style
- Builtins: `solid`, `checker`, `diagonal-l`, `diagonal-r`, `noise`

---

### Stamp

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

**Rules:**

- `glyph` is the character typed in shapes/prefabs/maps
- Body defines pixels; dimensions are the stamp size
- `$` = edge color, `.` = fill color, `x` = transparent (configurable per style)
- Stamps can be any size; blended when drawn

---

### Style

```yaml
---
name: brick-red
palette: dungeon
---
+: corner
-: edge-h
|: edge-v
B: brick
" ": { brush: solid, color: $fill }
x: transparent
```

**Rules:**

- Maps glyphs to stamps or inline definitions
- `palette:` required; can include variant: `dungeon@light-mode`
- Inheritance: `inherits: other-style`
- Inline brush: `{ brush: name, color: $x }` or `{ brush: name, colors: [$a, $b] }`

---

### Shape

```yaml
---
name: wall-segment
tags: [wall, solid]
---
+--+
|BB|
|BB|
+--+
```

**Rules:**

- No style in definition — applied at render
- `tags` for metadata export
- Each character resolved via style's glyph mappings
- Output size = sum of stamp sizes (variable)

---

### Prefab

```yaml
---
name: tower
tags: [structure]
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

**Rules:**

- Body is ASCII grid of placement
- Legend after second `---` maps characters to shape/prefab names
- Whitespace is literal positioning
- Can reference other prefabs (nested)

---

### Map

```yaml
---
name: level-1
tags: [level, dungeon]
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

### Builtin Brushes

|Name|Pattern|Description|
|---|---|---|
|`solid`|—|Single color fill|
|`checker`|`AB`/`BA`|2×2 checkerboard|
|`diagonal-r`|`AB`/`BA` offset|Diagonal lines (/)|
|`diagonal-l`|`BA`/`AB` offset|Diagonal lines ()|
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

### Builtin Style

```yaml
---
name: default
palette: default
---
# Uses all builtin stamps with builtin palette
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
2. Palette loading and color resolution
3. Stamp loading and pixel grid parsing
4. Style loading and glyph→stamp binding
5. Shape rendering (ASCII → pixels via style)
6. PNG output (single shape)

**Phase 2: Composition**

1. Prefab loading and shape placement
2. Map loading (same as prefab, larger)
3. Sprite sheet packing
4. JSON metadata export

**Phase 3: Variants & Targets**

1. Palette variants (`@light-mode`)
2. Style inheritance
3. Target profiles (PICO-8 format, scaling, etc.)
4. Build command with `--target`, `--variant`, `--style` flags

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
    colors: HashMap<String, Rgba>,
    variants: HashMap<String, HashMap<String, Rgba>>,
}

struct Stamp {
    glyph: String,
    pixels: Vec<Vec<PixelType>>, // Edge, Fill, Transparent, Color(idx)
}

struct Style {
    palette: String,
    variant: Option<String>,
    glyphs: HashMap<char, StampRef>,
}

struct Shape {
    name: String,
    tags: Vec<String>,
    grid: Vec<Vec<char>>, // resolved via style at render time
}
```

**Rendering pipeline:**

```
Shape.grid + Style → Vec<Vec<Stamp>> → Vec<Vec<Rgba>> → Image
```

Each cell in the grid maps to a stamp, stamps expand to pixels, pixels get final colors from palette.
