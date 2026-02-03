# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-02-02

### Added

- Parser infrastructure for markdown-style definition files
  - Document splitter for multi-definition files
  - YAML frontmatter extraction with source span tracking
  - Code block body extraction (```px fences)
  - Legend section parsing (glyph to reference mappings)
  - Source location tracking for error messages
- `Span`, `Spanned<T>`, and `Location` types for source positions
- `RawDocument` type for parsed documents
- `LegendValue` enum for simple and complex legend entries

## [0.1.0] - 2026-02-02

### Added

- Initial project structure
- CLI skeleton with `px build` and `px validate` commands (stubs)
- Error handling with miette diagnostics
- Test fixtures linked to examples
