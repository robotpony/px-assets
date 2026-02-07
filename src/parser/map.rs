//! Map file parser.
//!
//! Parses `.map.md` files into `Map` instances.
//! Structurally identical to prefab parsing - legend entries must be
//! simple name references (not brush/fill).

use std::collections::HashMap;

use crate::error::{PxError, Result};
use crate::parser::shape::parse_grid;
use crate::parser::types::LegendValue;
use crate::parser::{parse_documents, RawDocument};
use crate::types::Map;

/// Parse a map file into one or more maps.
///
/// Each document in the file becomes a separate map.
pub fn parse_map_file(source: &str) -> Result<Vec<Map>> {
    let documents = parse_documents(source)?;

    documents.into_iter().map(parse_map_document).collect()
}

/// Parse a single map document.
fn parse_map_document(doc: RawDocument) -> Result<Map> {
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

    let legend = convert_map_legend(&name, doc.legend)?;

    Ok(Map::with_scale(name, tags, grid, legend, scale))
}

/// Convert parser legend to map legend (only simple references allowed).
fn convert_map_legend(
    map_name: &str,
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
                        "Map '{}': legend entry '{}' must be a simple name reference, \
                         not a brush/fill binding",
                        map_name, glyph
                    ),
                    help: Some("Use `G: shape-name` format in map legends".to_string()),
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
    fn test_parse_simple_map() {
        let source = r#"---
name: dungeon-1
---

```px
WWWW
W..W
W..W
WWDW
```

---
W: wall-segment
D: door
.: floor
"#;

        let maps = parse_map_file(source).unwrap();
        assert_eq!(maps.len(), 1);

        let map = &maps[0];
        assert_eq!(map.name, "dungeon-1");
        assert_eq!(map.width(), 4);
        assert_eq!(map.height(), 4);
        assert_eq!(map.get_legend('W'), Some("wall-segment"));
        assert_eq!(map.get_legend('D'), Some("door"));
        assert_eq!(map.get_legend('.'), Some("floor"));
    }

    #[test]
    fn test_parse_map_with_tags() {
        let source = r##"---
name: level-1
tags: "#dungeon #dark"
---

```px
WW
WW
```

---
W: wall
"##;

        let maps = parse_map_file(source).unwrap();
        let map = &maps[0];

        assert_eq!(map.tags, vec!["dungeon", "dark"]);
    }

    #[test]
    fn test_parse_map_with_empty() {
        let source = r#"---
name: sparse-map
---

```px
W.W
...
W.W
```

---
W: wall
.: empty
"#;

        let maps = parse_map_file(source).unwrap();
        let map = &maps[0];

        assert_eq!(map.width(), 3);
        assert_eq!(map.height(), 3);
        assert_eq!(map.get_legend('.'), Some("empty"));
    }

    #[test]
    fn test_parse_multiple_maps() {
        let source = r#"---
name: map-a
---

```px
AB
```

---
A: shape-a
B: shape-b

---
name: map-b
---

```px
CD
```

---
C: shape-c
D: shape-d
"#;

        let maps = parse_map_file(source).unwrap();
        assert_eq!(maps.len(), 2);
        assert_eq!(maps[0].name, "map-a");
        assert_eq!(maps[1].name, "map-b");
    }

    #[test]
    fn test_parse_map_rejects_complex_legend() {
        let source = r#"---
name: bad-map
---

```px
~~
```

---
~: { fill: checker, A: $edge, B: $fill }
"#;

        let result = parse_map_file(source);
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("simple name reference"));
    }

    #[test]
    fn test_parse_map_with_scale() {
        let source = r#"---
name: scaled-map
scale: 4
---

```px
A
```

---
A: some-shape
"#;

        let maps = parse_map_file(source).unwrap();
        let map = &maps[0];

        assert_eq!(map.scale, Some(4));
    }

    #[test]
    fn test_parse_map_no_body() {
        let source = r#"---
name: empty-map
---
"#;

        let maps = parse_map_file(source).unwrap();
        let map = &maps[0];

        assert_eq!(map.width(), 1);
        assert_eq!(map.height(), 1);
    }
}
