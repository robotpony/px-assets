# Tetris example

A falling-block puzzle game sprite set demonstrating colour expressions, multi-colour sprites, and prefab composition.

## Build

```bash
# Individual sprites + map
px build examples/tetris/ -o examples/tetris/dist --shader tetris --scale 4

# Sprite sheet
px build examples/tetris/ -o examples/tetris/dist --shader tetris --scale 4 --sheet
```

## Assets

**Palette** (`tetris.palette.md`) - 7 piece colours with `lighten()` and `darken()` expressions for beveled shading. Each colour has four tones: highlight, light edge, fill, and dark edge.

**Blocks** (`blocks.shape.md`) - 8x8 beveled blocks for each piece type (I/O/T/S/Z/J/L). Uses the multi-colour technique with `{ stamp: solid, A: $colour }` legend entries to produce 4-tone shading per block.

**UI** (`ui.shape.md`) - Wall border tile, empty playfield cell (both beveled in grey tones), and a transparent empty tile.

**Prefabs** (`pieces.prefab.md`) - All 7 tetromino shapes composed from individual block sprites. Transparent cells use the `empty` shape.

**Map** (`playfield.map.md`) - Full 10x20 Tetris playfield with walls, showing a mid-game state: T-piece dropping, J/S/O/L/I/Z pieces stacked at the bottom.

## Techniques

### Colour expressions

The palette defines 4 shades per piece colour using `lighten()` and `darken()`:

```
$cyan: #00E5FF
$cyan-hi: lighten($cyan, 40%)
$cyan-light: lighten($cyan, 20%)
$cyan-dark: darken($cyan, 30%)
```

### Beveled blocks

Each 8x8 block uses 4 legend entries for the 3D beveled look:

```
L: { stamp: solid, A: $cyan-light }    # top/left edge
H: { stamp: solid, A: $cyan-hi }       # specular highlight
F: { stamp: solid, A: $cyan }          # main fill
D: { stamp: solid, A: $cyan-dark }     # bottom/right shadow
```

The grid pattern creates a light-to-dark gradient from top-left to bottom-right:

```px
LLLLLLLD
LHHFFFFD
LHFFFFFD
LFFFFFFD
LFFFFFFD
LFFFFFFD
LFFFFFFD
DDDDDDDD
```
