//! Stamp file parser.
//!
//! Parses `.stamp.md` files into `Stamp` instances.

use crate::error::{PxError, Result};
use crate::parser::{parse_documents, RawDocument};
use crate::types::{PixelToken, Stamp};

/// Parse a stamp file into one or more stamps.
///
/// Each document in the file becomes a separate stamp.
pub fn parse_stamp_file(source: &str) -> Result<Vec<Stamp>> {
    let documents = parse_documents(source)?;

    documents
        .into_iter()
        .map(parse_stamp_document)
        .collect()
}

/// Parse a single stamp document.
fn parse_stamp_document(doc: RawDocument) -> Result<Stamp> {
    let name = doc.name.value.clone();

    // Get glyph from frontmatter (optional)
    let glyph = parse_glyph(&doc)?;

    // Parse pixel grid from body
    let pixels = if let Some(body) = &doc.body {
        parse_pixel_grid(&body.value)?
    } else {
        // Empty stamp - default to single fill pixel
        vec![vec![PixelToken::Fill]]
    };

    if pixels.is_empty() {
        return Err(PxError::Parse {
            message: format!("Stamp '{}' has no pixels", name),
            help: Some("Add pixel content in a ```px block".to_string()),
        });
    }

    Ok(Stamp::new(name, glyph, pixels))
}

/// Parse the glyph from frontmatter.
fn parse_glyph(doc: &RawDocument) -> Result<Option<char>> {
    let glyph_value = doc.get_frontmatter_str("glyph");

    match glyph_value {
        None => Ok(None),
        Some(s) => {
            // YAML parses `glyph: " "` as a single space string
            // Don't trim - we want to preserve spaces

            // Handle quoted characters in the raw value
            // If YAML already stripped quotes, the value is ready
            let c = if s.is_empty() {
                return Err(PxError::Parse {
                    message: format!("Empty glyph for stamp '{}'", doc.name.value),
                    help: Some("Glyph must be a single character".to_string()),
                });
            } else if (s.starts_with('"') && s.ends_with('"'))
                || (s.starts_with('\'') && s.ends_with('\''))
            {
                // Explicit quotes in value (rare, but handle it)
                let inner = &s[1..s.len() - 1];
                if inner.is_empty() {
                    return Err(PxError::Parse {
                        message: format!("Empty glyph for stamp '{}'", doc.name.value),
                        help: Some("Glyph must be a single character".to_string()),
                    });
                }
                inner.chars().next().unwrap()
            } else {
                // Use first character directly
                s.chars().next().unwrap()
            };

            Ok(Some(c))
        }
    }
}

/// Parse a pixel grid from the body content.
fn parse_pixel_grid(body: &str) -> Result<Vec<Vec<PixelToken>>> {
    let mut rows: Vec<Vec<PixelToken>> = Vec::new();
    let mut max_width = 0;

    for line in body.lines() {
        // Skip empty lines at start/end, but preserve them in middle
        if rows.is_empty() && line.trim().is_empty() {
            continue;
        }

        let row: Vec<PixelToken> = line
            .chars()
            .map(|c| PixelToken::from_char(c).unwrap_or(PixelToken::Transparent))
            .collect();

        if !row.is_empty() {
            max_width = max_width.max(row.len());
            rows.push(row);
        }
    }

    // Normalize row widths (pad shorter rows with transparent)
    for row in &mut rows {
        while row.len() < max_width {
            row.push(PixelToken::Transparent);
        }
    }

    // Trim trailing empty rows
    while rows.last().map_or(false, |r| r.iter().all(|t| *t == PixelToken::Transparent)) {
        rows.pop();
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_stamp() {
        let source = r#"---
name: brick
glyph: B
---

```px
$$$$
$..$
$..$
$$$$
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        assert_eq!(stamps.len(), 1);

        let stamp = &stamps[0];
        assert_eq!(stamp.name, "brick");
        assert_eq!(stamp.glyph, Some('B'));
        assert_eq!(stamp.width(), 4);
        assert_eq!(stamp.height(), 4);

        // Check corners are edge
        assert_eq!(stamp.get(0, 0), Some(PixelToken::Edge));
        assert_eq!(stamp.get(3, 3), Some(PixelToken::Edge));

        // Check interior is fill
        assert_eq!(stamp.get(1, 1), Some(PixelToken::Fill));
        assert_eq!(stamp.get(2, 2), Some(PixelToken::Fill));
    }

    #[test]
    fn test_parse_stamp_no_glyph() {
        let source = r#"---
name: custom
---

```px
$
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        assert_eq!(stamps.len(), 1);
        assert_eq!(stamps[0].glyph, None);
    }

    #[test]
    fn test_parse_stamp_quoted_glyph() {
        let source = r#"---
name: pipe
glyph: "|"
---

```px
$
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        assert_eq!(stamps[0].glyph, Some('|'));
    }

    #[test]
    fn test_parse_stamp_space_glyph() {
        let source = r#"---
name: space
glyph: " "
---

```px
.
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        assert_eq!(stamps[0].glyph, Some(' '));
    }

    #[test]
    fn test_parse_multiple_stamps() {
        let source = r#"---
name: corner
glyph: +
---

```px
$
```

---
name: fill
glyph: .
---

```px
.
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        assert_eq!(stamps.len(), 2);
        assert_eq!(stamps[0].name, "corner");
        assert_eq!(stamps[1].name, "fill");
    }

    #[test]
    fn test_parse_stamp_with_transparency() {
        let source = r#"---
name: arrow
glyph: ">"
---

```px
x$x
$$x
x$x
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        let stamp = &stamps[0];

        assert_eq!(stamp.glyph, Some('>'));
        assert_eq!(stamp.width(), 3);
        assert_eq!(stamp.height(), 3);
        assert_eq!(stamp.get(0, 0), Some(PixelToken::Transparent));
        assert_eq!(stamp.get(1, 0), Some(PixelToken::Edge));
        assert_eq!(stamp.get(0, 1), Some(PixelToken::Edge));
    }

    #[test]
    fn test_parse_pixel_grid_normalizes_width() {
        let grid = parse_pixel_grid("$\n$$$\n$").unwrap();

        // All rows should be same width (3)
        assert_eq!(grid[0].len(), 3);
        assert_eq!(grid[1].len(), 3);
        assert_eq!(grid[2].len(), 3);

        // First row: edge, transparent, transparent
        assert_eq!(grid[0][0], PixelToken::Edge);
        assert_eq!(grid[0][1], PixelToken::Transparent);
        assert_eq!(grid[0][2], PixelToken::Transparent);
    }

    #[test]
    fn test_parse_unknown_char_becomes_transparent() {
        let grid = parse_pixel_grid("$?$").unwrap();

        assert_eq!(grid[0][0], PixelToken::Edge);
        assert_eq!(grid[0][1], PixelToken::Transparent); // ? -> transparent
        assert_eq!(grid[0][2], PixelToken::Edge);
    }

    #[test]
    fn test_parse_default_stamps() {
        // Parse the actual default stamps file format
        let source = r#"---
name: corner
glyph: +
---

```px
$
```

---
name: edge-h
glyph: "-"
---

```px
$
```
"#;

        let stamps = parse_stamp_file(source).unwrap();
        assert_eq!(stamps.len(), 2);

        let corner = &stamps[0];
        assert_eq!(corner.name, "corner");
        assert_eq!(corner.glyph, Some('+'));
        assert_eq!(corner.size(), (1, 1));
        assert_eq!(corner.get(0, 0), Some(PixelToken::Edge));

        let edge_h = &stamps[1];
        assert_eq!(edge_h.name, "edge-h");
        assert_eq!(edge_h.glyph, Some('-'));
    }
}
