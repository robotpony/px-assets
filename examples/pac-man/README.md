# Pac-Man example sprites

A full Pac-Man arcade sprite set demonstrating multi-colour sprites via brush colour bindings. Each glyph in the grid maps to one colour through the `solid` builtin brush with a single colour binding, giving unlimited colours per shape.

## Build

```bash
# Build all assets to PNG (4x scale)
px build examples/pac-man/ -o /tmp/pac-man --scale 4

# Validate (zero errors expected)
px validate examples/pac-man/

# Pack sprites into a sheet
px build examples/pac-man/ -o /tmp/pac-man --sheet --scale 4
```

## Technique

Standard shapes get two colours (edge + fill from the palette). For multi-colour sprites, brush colour bindings override this:

```
Y: { stamp: solid, A: $yellow }
W: { stamp: solid, A: $white }
B: { stamp: solid, A: $dark-blue }
```

Each glyph becomes a single pixel of the bound colour. The palette sets `$edge: $blue` and `$fill: $black`, so builtin glyphs (`#` = blue, `.` = black, `x` = transparent) handle maze walls without legend entries.

## Files

- pac-man.palette.md (arcade colours: yellow, blue, red, pink, cyan, orange)
- pac-man.shader.md (palette binding)
- pacman.shape.md (Pac-Man: 4 directions x 2 mouth states + closed, 16x16)
- ghosts.shape.md (blinky, pinky, inky, clyde, frightened, eyes-only, 16x16)
- collectibles.shape.md (cherry, 15x15)
- maze-walls.shape.md (wall tiles, gate, path, dot, pellet, 8x8)
- fills.shape.md (empty transparent tile)
- maze.map.md (28x21 tile maze with ghost pen and power pellets)
- test.target.md (PNG, scale 4, metadata on)

### Colours

```
#000000 black
#2121DE blue
#FFFF00 yellow
#FF0000 red
#FFB8FF pink
#00FFDE cyan
#FFB852 orange
#2121FF dark-blue
#FFCC99 cream
#00FF00 green
#FFFFFF white
```
