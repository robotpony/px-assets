//! Shape file parser.
//!
//! Parses `.shape.md` files into `Shape` instances.

use std::collections::HashMap;

use crate::error::Result;
use crate::parser::types::LegendValue;
use crate::parser::{parse_documents, RawDocument};
use crate::types::{LegendEntry, Shape};

/// Parse a shape file into one or more shapes.
///
/// Each document in the file becomes a separate shape.
pub fn parse_shape_file(source: &str) -> Result<Vec<Shape>> {
    let documents = parse_documents(source)?;

    documents.into_iter().map(parse_shape_document).collect()
}

/// Parse a single shape document.
fn parse_shape_document(doc: RawDocument) -> Result<Shape> {
    let name = doc.name.value.clone();

    // Get tags
    let tags = doc.get_tags();

    // Get scale from frontmatter
    let scale = doc
        .get_frontmatter("scale")
        .and_then(|v| v.value.as_u64())
        .map(|s| s as u32);

    // Parse ASCII grid from body
    let grid = if let Some(body) = &doc.body {
        parse_grid(&body.value)
    } else {
        vec![vec!['x']] // Default to single transparent cell
    };

    // Convert legend
    let legend = convert_legend(doc.legend);

    Ok(Shape::with_scale(name, tags, grid, legend, scale))
}

/// Parse the ASCII grid from body content.
fn parse_grid(body: &str) -> Vec<Vec<char>> {
    let mut rows: Vec<Vec<char>> = Vec::new();
    let mut max_width = 0;

    for line in body.lines() {
        // Collect all characters (preserve spaces within the line)
        let row: Vec<char> = line.chars().collect();

        // Track max width (but don't skip empty lines in the middle)
        if !row.is_empty() || !rows.is_empty() {
            max_width = max_width.max(row.len());
            rows.push(row);
        }
    }

    // Trim trailing empty rows
    while rows.last().map_or(false, |r| r.is_empty()) {
        rows.pop();
    }

    // Normalize row widths (pad shorter rows with spaces)
    for row in &mut rows {
        while row.len() < max_width {
            row.push(' ');
        }
    }

    // If completely empty, return minimal grid
    if rows.is_empty() {
        rows.push(vec!['x']);
    }

    rows
}

/// Convert parser LegendValue to type LegendEntry.
fn convert_legend(
    legend: Option<HashMap<char, crate::parser::span::Spanned<LegendValue>>>,
) -> HashMap<char, LegendEntry> {
    let Some(legend) = legend else {
        return HashMap::new();
    };

    legend
        .into_iter()
        .map(|(glyph, spanned)| {
            let entry = match spanned.value {
                LegendValue::Reference(name) => LegendEntry::StampRef(name),
                LegendValue::Complex {
                    name,
                    fill,
                    bindings,
                } => {
                    // bindings is already HashMap<char, String>
                    if fill {
                        LegendEntry::Fill { name, bindings }
                    } else {
                        LegendEntry::BrushRef { name, bindings }
                    }
                }
            };
            (glyph, entry)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_shape() {
        let source = r#"---
name: test
---

```px
+--+
|..|
+--+
```
"#;

        let shapes = parse_shape_file(source).unwrap();
        assert_eq!(shapes.len(), 1);

        let shape = &shapes[0];
        assert_eq!(shape.name, "test");
        assert_eq!(shape.width(), 4);
        assert_eq!(shape.height(), 3);
        assert_eq!(shape.get(0, 0), Some('+'));
        assert_eq!(shape.get(1, 1), Some('.'));
    }

    #[test]
    fn test_parse_shape_with_tags() {
        // Tags with # must be quoted in YAML (# starts a comment otherwise)
        let source = r##"---
name: wall
tags: "#wall #solid"
---

```px
###
```
"##;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        assert_eq!(shape.tags, vec!["wall", "solid"]);
    }

    #[test]
    fn test_parse_shape_with_legend() {
        let source = r#"---
name: test
---

```px
BB
BB
```

---
B: brick
"#;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        assert!(shape.has_legend('B'));
        if let Some(LegendEntry::StampRef(name)) = shape.get_legend('B') {
            assert_eq!(name, "brick");
        } else {
            panic!("Expected StampRef");
        }
    }

    #[test]
    fn test_parse_multiple_shapes() {
        let source = r#"---
name: shape-a
---

```px
AA
```

---
name: shape-b
---

```px
BB
```
"#;

        let shapes = parse_shape_file(source).unwrap();
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0].name, "shape-a");
        assert_eq!(shapes[1].name, "shape-b");
    }

    #[test]
    fn test_parse_shape_preserves_spaces() {
        let source = r#"---
name: player
---

```px
 oo
 ||
+  +
```
"#;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        assert_eq!(shape.width(), 4);
        assert_eq!(shape.get(0, 0), Some(' '));
        assert_eq!(shape.get(1, 0), Some('o'));
        assert_eq!(shape.get(1, 2), Some(' ')); // Middle space in "+  +"
    }

    #[test]
    fn test_parse_shape_normalizes_width() {
        let source = r#"---
name: test
---

```px
A
ABC
AB
```
"#;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        // All rows should be padded to width 3
        assert_eq!(shape.width(), 3);
        assert_eq!(shape.get(2, 0), Some(' ')); // Padded
        assert_eq!(shape.get(2, 1), Some('C')); // Original
        assert_eq!(shape.get(2, 2), Some(' ')); // Padded
    }

    #[test]
    fn test_parse_shape_with_complex_legend() {
        let source = r#"---
name: test
---

```px
~~
```

---
~: { fill: checker, A: $edge, B: $fill }
"#;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        if let Some(LegendEntry::Fill { name, bindings }) = shape.get_legend('~') {
            assert_eq!(name, "checker");
            assert_eq!(bindings.get(&'A'), Some(&"$edge".to_string()));
            assert_eq!(bindings.get(&'B'), Some(&"$fill".to_string()));
        } else {
            panic!("Expected Fill entry");
        }
    }

    #[test]
    fn test_parse_real_player_shape() {
        // Tags with # must be quoted in YAML (# starts a comment otherwise)
        let source = r##"---
name: player-stand
tags: "#player"
---

```px
 oo
 ||
+  +
|  |
```

---
o: fill
"##;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        assert_eq!(shape.name, "player-stand");
        assert_eq!(shape.tags, vec!["player"]);
        assert_eq!(shape.size(), (4, 4));

        // Check legend
        if let Some(LegendEntry::StampRef(name)) = shape.get_legend('o') {
            assert_eq!(name, "fill");
        } else {
            panic!("Expected StampRef for 'o'");
        }
    }

    #[test]
    fn test_parse_shape_with_scale() {
        let source = r#"---
name: scaled-shape
scale: 4
---

```px
##
##
```
"#;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        assert_eq!(shape.name, "scaled-shape");
        assert_eq!(shape.scale, Some(4));
    }

    #[test]
    fn test_parse_shape_without_scale() {
        let source = r#"---
name: no-scale
---

```px
#
```
"#;

        let shapes = parse_shape_file(source).unwrap();
        let shape = &shapes[0];

        assert_eq!(shape.scale, None);
    }
}
