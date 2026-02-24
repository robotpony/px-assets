# CT-91 (CINOTRIS)

A retro Tetris-style game sprite set recreated as px definition files, based on reference sprite sheets from 2014.

## Source material

Two reference PNGs with 2x pixel doubling (each logical pixel is a 2x2 block):

- `CT_PRTS.png` (128x512) - gems, font, tiles, palette strip
- `CT_ANI.png` (128x145) - title text, UI labels

Both use a 16-colour CGA/EGA-era indexed palette.

## Palette

The 16 colours were extracted using `px palette`:

```bash
px palette CT_PRTS.png
```

Standard CGA palette: black, dark-red, dark-green, olive, dark-blue, dark-magenta, dark-cyan, silver, grey, red, green, yellow, blue, magenta, cyan, white.

## Sprites

| File | Count | Description |
|------|-------|-------------|
| `gems.shape.md` | 8 | Diamond sprites in 8 colours (8x8 each) |
| `font-digits.shape.md` | 10 | Digits 0-9 with 3-colour gradient |
| `font-symbols.shape.md` | 7 | Symbols: + - / x +/- ? ! |
| `font-upper.shape.md` | 26 | Uppercase A-Z |
| `font-punctuation.shape.md` | 6 | Punctuation: . , : ; ( ) |
| `tiles.shape.md` | 9 | Block tiles: bordered, solid, striped, inset variants |
| `title-letters.shape.md` | 21 | CINOTRIS letters in green, yellow, and red |
| `ui-labels.shape.md` | 4 | UI elements: MM, A, D, X |
| `title.prefab.md` | 3 | Title text rows composed from letter shapes |

## px features demonstrated

1. **`px palette`** extracts the colour set from reference art
2. **Multi-colour shapes** using `{ stamp: solid, A: $colour }` legend bindings
3. **3-colour gradient font** with row-based colour zones (olive/dark-green/dark-blue)
4. **Prefab composition** assembling title text from individual letter shapes
5. **Sprite sheets** packing all sprites into a single atlas
6. **Target profiles** bundling output settings

## Build

```bash
# Individual sprites at 2x scale
px build examples/ct-91/ --scale 2

# Sprite sheet output
px build examples/ct-91/ --sheet --scale 2 -o examples/ct-91/dist
```
