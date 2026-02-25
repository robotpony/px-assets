//! Brush file parser.
//!
//! Parses `.brush.md` files into `Brush` instances.

use crate::error::{PxError, Result};
use crate::parser::{parse_documents, RawDocument};
use crate::types::Brush;

/// Parse a brush file into one or more brushes.
///
/// Each document in the file becomes a separate brush.
pub fn parse_brush_file(source: &str) -> Result<Vec<Brush>> {
    let documents = parse_documents(source)?;

    documents.into_iter().map(parse_brush_document).collect()
}

/// Parse a single brush document.
fn parse_brush_document(doc: RawDocument) -> Result<Brush> {
    let name = doc.name.value.clone();

    // Parse pattern grid from body
    let pattern = if let Some(body) = &doc.body {
        parse_pattern_grid(&body.value)?
    } else {
        // Empty brush - default to single A token
        vec![vec!['A']]
    };

    if pattern.is_empty() {
        return Err(PxError::Parse {
            message: format!("Brush '{}' has no pattern", name),
            help: Some("Add pattern content in a ```px block".to_string()),
        });
    }

    Ok(Brush::new(name, pattern))
}

/// Parse a pattern grid from the body content.
///
/// Accepts uppercase letters A-Z as tokens. Other characters are preserved
/// but typically only A and B are used.
fn parse_pattern_grid(body: &str) -> Result<Vec<Vec<char>>> {
    let mut rows: Vec<Vec<char>> = Vec::new();
    let mut max_width = 0;

    for line in body.lines() {
        // Skip empty lines at start/end
        if rows.is_empty() && line.trim().is_empty() {
            continue;
        }

        // Collect characters (skip trailing whitespace)
        let row: Vec<char> = line.trim_end().chars().collect();

        if !row.is_empty() {
            max_width = max_width.max(row.len());
            rows.push(row);
        }
    }

    // Normalize row widths (pad shorter rows with first token or 'A')
    let default_token = rows
        .first()
        .and_then(|r| r.first())
        .copied()
        .unwrap_or('A');

    for row in &mut rows {
        while row.len() < max_width {
            row.push(default_token);
        }
    }

    // Trim trailing empty rows
    while rows
        .last()
        .map_or(false, |r| r.iter().all(|&c| c.is_whitespace()))
    {
        rows.pop();
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_brush() {
        let source = r#"---
name: checker
---

```px
AB
BA
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        assert_eq!(brushes.len(), 1);

        let brush = &brushes[0];
        assert_eq!(brush.name, "checker");
        assert_eq!(brush.width(), 2);
        assert_eq!(brush.height(), 2);

        assert_eq!(brush.get(0, 0), Some('A'));
        assert_eq!(brush.get(1, 0), Some('B'));
        assert_eq!(brush.get(0, 1), Some('B'));
        assert_eq!(brush.get(1, 1), Some('A'));
    }

    #[test]
    fn test_parse_solid_brush() {
        let source = r#"---
name: solid
---

```px
A
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        assert_eq!(brushes.len(), 1);

        let brush = &brushes[0];
        assert_eq!(brush.name, "solid");
        assert_eq!(brush.size(), (1, 1));
        assert_eq!(brush.get(0, 0), Some('A'));
    }

    #[test]
    fn test_parse_multiple_brushes() {
        let source = r#"---
name: solid
---

```px
A
```

---
name: checker
---

```px
AB
BA
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        assert_eq!(brushes.len(), 2);
        assert_eq!(brushes[0].name, "solid");
        assert_eq!(brushes[1].name, "checker");
    }

    #[test]
    fn test_parse_h_line_brush() {
        let source = r#"---
name: h-line
---

```px
A
B
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        let brush = &brushes[0];

        assert_eq!(brush.size(), (1, 2));
        assert_eq!(brush.get(0, 0), Some('A'));
        assert_eq!(brush.get(0, 1), Some('B'));
    }

    #[test]
    fn test_parse_v_line_brush() {
        let source = r#"---
name: v-line
---

```px
AB
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        let brush = &brushes[0];

        assert_eq!(brush.size(), (2, 1));
        assert_eq!(brush.get(0, 0), Some('A'));
        assert_eq!(brush.get(1, 0), Some('B'));
    }

    #[test]
    fn test_parse_noise_brush() {
        let source = r#"---
name: noise
---

```px
ABBA
BAAB
AABB
BBAA
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        let brush = &brushes[0];

        assert_eq!(brush.size(), (4, 4));
        assert_eq!(brush.get(0, 0), Some('A'));
        assert_eq!(brush.get(1, 0), Some('B'));
        assert_eq!(brush.get(2, 0), Some('B'));
        assert_eq!(brush.get(3, 0), Some('A'));
    }

    #[test]
    fn test_parse_brush_no_body() {
        let source = r#"---
name: empty
---
"#;

        let brushes = parse_brush_file(source).unwrap();
        let brush = &brushes[0];

        // Should default to single A
        assert_eq!(brush.size(), (1, 1));
        assert_eq!(brush.get(0, 0), Some('A'));
    }

    #[test]
    fn test_parse_pattern_normalizes_width() {
        let grid = parse_pattern_grid("A\nABC\nA").unwrap();

        // All rows should be same width (3)
        assert_eq!(grid[0].len(), 3);
        assert_eq!(grid[1].len(), 3);
        assert_eq!(grid[2].len(), 3);

        // First row: A, A, A (padded with 'A')
        assert_eq!(grid[0], vec!['A', 'A', 'A']);
        // Second row: A, B, C
        assert_eq!(grid[1], vec!['A', 'B', 'C']);
        // Third row: A, A, A (padded)
        assert_eq!(grid[2], vec!['A', 'A', 'A']);
    }

    #[test]
    fn test_parse_default_brushes() {
        // Parse the actual default brushes file format
        let source = r#"---
name: solid
---

```px
A
```

---
name: checker
---

```px
AB
BA
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        assert_eq!(brushes.len(), 2);

        let solid = &brushes[0];
        assert_eq!(solid.name, "solid");
        assert_eq!(solid.size(), (1, 1));

        let checker = &brushes[1];
        assert_eq!(checker.name, "checker");
        assert_eq!(checker.size(), (2, 2));
    }

    #[test]
    fn test_parse_brush_lowercase_tokens() {
        let source = r#"---
name: lower
---

```px
ab
ba
```
"#;

        let brushes = parse_brush_file(source).unwrap();
        let brush = &brushes[0];

        // Lowercase tokens are preserved as-is
        assert_eq!(brush.size(), (2, 2));
        assert_eq!(brush.get(0, 0), Some('a'));
        assert_eq!(brush.get(1, 0), Some('b'));
        assert_eq!(brush.get(0, 1), Some('b'));
        assert_eq!(brush.get(1, 1), Some('a'));
    }

    #[test]
    fn test_parse_brush_empty_code_block() {
        let source = "---\nname: empty\n---\n\n```px\n```\n";

        // Empty code block → empty body string → no pattern → error
        let result = parse_brush_file(source);
        assert!(result.is_err());
    }
}
