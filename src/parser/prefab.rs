//! Prefab file parser.
//!
//! Parses `.prefab.md` files into `Prefab` instances.
//! Legend entries must be simple name references (not brush/fill).

use std::collections::HashMap;

use crate::error::{PxError, Result};
use crate::parser::shape::parse_grid;
use crate::parser::types::LegendValue;
use crate::parser::{parse_documents, RawDocument};
use crate::types::Prefab;

/// Parse a prefab file into one or more prefabs.
///
/// Each document in the file becomes a separate prefab.
pub fn parse_prefab_file(source: &str) -> Result<Vec<Prefab>> {
    let documents = parse_documents(source)?;

    documents.into_iter().map(parse_prefab_document).collect()
}

/// Parse a single prefab document.
fn parse_prefab_document(doc: RawDocument) -> Result<Prefab> {
    let name = doc.name.value.clone();
    let tags = doc.get_tags();

    let scale = doc
        .get_frontmatter("scale")
        .and_then(|v| v.value.as_u64())
        .map(|s| s as u32);

    let grid = if let Some(body) = &doc.body {
        parse_grid(&body.value)
    } else {
        vec![vec![' ']]
    };

    let legend = convert_prefab_legend(&name, doc.legend)?;

    Ok(Prefab::with_scale(name, tags, grid, legend, scale))
}

/// Convert parser legend to prefab legend (only simple references allowed).
fn convert_prefab_legend(
    prefab_name: &str,
    legend: Option<HashMap<char, crate::parser::span::Spanned<LegendValue>>>,
) -> Result<HashMap<char, String>> {
    let Some(legend) = legend else {
        return Ok(HashMap::new());
    };

    let mut result = HashMap::new();

    for (glyph, spanned) in legend {
        match spanned.value {
            LegendValue::Reference(name) => {
                result.insert(glyph, name);
            }
            LegendValue::Complex { .. } => {
                return Err(PxError::Parse {
                    message: format!(
                        "Prefab '{}': legend entry '{}' must be a simple name reference, \
                         not a brush/fill binding",
                        prefab_name, glyph
                    ),
                    help: Some("Use `G: shape-name` format in prefab legends".to_string()),
                });
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_prefab() {
        let source = r#"---
name: tower
---

```px
R
W
D
```

---
R: roof
W: wall-segment
D: door
"#;

        let prefabs = parse_prefab_file(source).unwrap();
        assert_eq!(prefabs.len(), 1);

        let prefab = &prefabs[0];
        assert_eq!(prefab.name, "tower");
        assert_eq!(prefab.width(), 1);
        assert_eq!(prefab.height(), 3);
        assert_eq!(prefab.get_legend('R'), Some("roof"));
        assert_eq!(prefab.get_legend('W'), Some("wall-segment"));
        assert_eq!(prefab.get_legend('D'), Some("door"));
    }

    #[test]
    fn test_parse_prefab_with_tags() {
        let source = r##"---
name: castle
tags: "#building #large"
---

```px
TT
WW
```

---
T: tower
W: wall
"##;

        let prefabs = parse_prefab_file(source).unwrap();
        let prefab = &prefabs[0];

        assert_eq!(prefab.tags, vec!["building", "large"]);
    }

    #[test]
    fn test_parse_prefab_with_spaces() {
        let source = r#"---
name: spaced
---

```px
A B
 C
```

---
A: left
B: right
C: center
"#;

        let prefabs = parse_prefab_file(source).unwrap();
        let prefab = &prefabs[0];

        assert_eq!(prefab.width(), 3);
        assert_eq!(prefab.height(), 2);
        assert_eq!(prefab.get(0, 0), Some('A'));
        assert_eq!(prefab.get(1, 0), Some(' '));
        assert_eq!(prefab.get(2, 0), Some('B'));
    }

    #[test]
    fn test_parse_multiple_prefabs() {
        let source = r#"---
name: prefab-a
---

```px
AB
```

---
A: shape-a
B: shape-b

---
name: prefab-b
---

```px
CD
```

---
C: shape-c
D: shape-d
"#;

        let prefabs = parse_prefab_file(source).unwrap();
        assert_eq!(prefabs.len(), 2);
        assert_eq!(prefabs[0].name, "prefab-a");
        assert_eq!(prefabs[1].name, "prefab-b");
    }

    #[test]
    fn test_parse_prefab_rejects_complex_legend() {
        let source = r#"---
name: bad-prefab
---

```px
~~
```

---
~: { fill: checker, A: $edge, B: $fill }
"#;

        let result = parse_prefab_file(source);
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("simple name reference"));
    }

    #[test]
    fn test_parse_prefab_with_scale() {
        let source = r#"---
name: scaled-prefab
scale: 4
---

```px
A
```

---
A: some-shape
"#;

        let prefabs = parse_prefab_file(source).unwrap();
        let prefab = &prefabs[0];

        assert_eq!(prefab.scale, Some(4));
    }

    #[test]
    fn test_parse_prefab_no_body() {
        let source = r#"---
name: empty-prefab
---
"#;

        let prefabs = parse_prefab_file(source).unwrap();
        let prefab = &prefabs[0];

        // Default grid is a single space
        assert_eq!(prefab.width(), 1);
        assert_eq!(prefab.height(), 1);
    }
}
