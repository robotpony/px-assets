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

**Phase 1.2 complete** - Parser infrastructure.

- Document splitter for multi-definition files
- YAML frontmatter extraction
- Code block body extraction
- Legend section parsing
- Source location tracking for errors

See [PLAN.md](PLAN.md) for progress and [SPEC.md](SPEC.md) for the DSL specification.
