//! Document splitting and parsing.
//!
//! Files can contain multiple definitions, each starting with `---` followed
//! by YAML frontmatter containing at least `name:`.

use crate::error::{PxError, Result};

use super::body::extract_body;
use super::frontmatter::extract_frontmatter;
use super::legend::extract_legend;
use super::span::{Span, Spanned};
use super::types::RawDocument;

/// Parse a file containing one or more document definitions.
///
/// Each definition starts with `---` and YAML frontmatter containing `name:`.
pub fn parse_documents(source: &str) -> Result<Vec<RawDocument>> {
    let sections = split_documents(source);

    let mut documents = Vec::new();

    for (section, base_offset) in sections {
        let doc = parse_single_document(&section, base_offset)?;
        documents.push(doc);
    }

    if documents.is_empty() {
        return Err(PxError::Parse {
            message: "No documents found in file".to_string(),
            help: Some("Add a document with ---\\nname: my-name\\n---".to_string()),
        });
    }

    Ok(documents)
}

/// Split source into document sections.
///
/// Returns (section_content, base_offset) for each document.
fn split_documents(source: &str) -> Vec<(String, usize)> {
    let mut sections = Vec::new();
    let mut current_start = 0;
    let mut in_code_block = false;

    let lines: Vec<&str> = source.lines().collect();
    let mut offset = 0;

    for (i, line) in lines.iter().enumerate() {
        // Track code blocks to avoid splitting inside them
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
        }

        // Look for document boundary: --- followed by name:
        if !in_code_block && line.trim() == "---" && i > 0 {
            // Check if next line has name:
            if let Some(next_line) = lines.get(i + 1) {
                if next_line.trim().starts_with("name:") {
                    // This is a new document boundary
                    let section = &source[current_start..offset];
                    if !section.trim().is_empty() {
                        sections.push((section.to_string(), current_start));
                    }
                    current_start = offset;
                }
            }
        }

        offset += line.len() + 1; // +1 for newline
    }

    // Add final section
    let final_section = &source[current_start..];
    if !final_section.trim().is_empty() {
        sections.push((final_section.to_string(), current_start));
    }

    // If no sections found, treat entire file as one document
    if sections.is_empty() && !source.trim().is_empty() {
        sections.push((source.to_string(), 0));
    }

    sections
}

/// Parse a single document section.
fn parse_single_document(source: &str, base_offset: usize) -> Result<RawDocument> {
    // Extract frontmatter
    let frontmatter_result = extract_frontmatter(source, base_offset)?;

    // Get name from frontmatter (required)
    let name = frontmatter_result
        .values
        .get("name")
        .and_then(|v| v.value.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| PxError::Parse {
            message: "Document missing required 'name' field".to_string(),
            help: Some("Add name: my-name to frontmatter".to_string()),
        })?;

    let name_span = frontmatter_result
        .values
        .get("name")
        .map(|v| v.span)
        .unwrap_or(frontmatter_result.span);

    // Extract body (optional)
    let remaining = &source[frontmatter_result.content_start..];
    let remaining_offset = base_offset + frontmatter_result.content_start;

    let (body, legend_source_start) = match extract_body(remaining, remaining_offset) {
        Some(body_result) => (
            Some(body_result.content),
            frontmatter_result.content_start + body_result.content_end,
        ),
        None => (None, frontmatter_result.content_start),
    };

    // Extract legend (optional)
    let legend_source = &source[legend_source_start..];
    let legend_offset = base_offset + legend_source_start;

    let legend = match extract_legend(legend_source, legend_offset)? {
        Some(legend_result) => Some(legend_result.entries),
        None => None,
    };

    // Calculate document span
    let doc_span = Span::from_local_offsets(source, 0, source.len(), base_offset);

    Ok(RawDocument {
        name: Spanned::new(name, name_span),
        frontmatter: frontmatter_result.values,
        body,
        legend,
        span: doc_span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_document() {
        let source = r#"---
name: test-shape
tags: #player
---

```px
+--+
|..|
+--+
```

---
B: brick
"#;

        let docs = parse_documents(source).unwrap();

        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name.value, "test-shape");
        assert!(docs[0].body.is_some());
        assert_eq!(docs[0].body.as_ref().unwrap().value, "+--+\n|..|\n+--+");
        assert!(docs[0].legend.is_some());
        assert!(docs[0].legend.as_ref().unwrap().contains_key(&'B'));
    }

    #[test]
    fn test_parse_multiple_documents() {
        let source = r#"---
name: shape-a
---

```px
AB
```

---
A: stamp-one

---
name: shape-b
---

```px
XY
```

---
X: stamp-two
"#;

        let docs = parse_documents(source).unwrap();

        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name.value, "shape-a");
        assert_eq!(docs[1].name.value, "shape-b");

        // Each doc should have its own legend
        assert!(docs[0].legend.as_ref().unwrap().contains_key(&'A'));
        assert!(docs[1].legend.as_ref().unwrap().contains_key(&'X'));
    }

    #[test]
    fn test_parse_document_without_body() {
        let source = r#"---
name: palette
---
$dark: #1a1a2e
$light: #4a4a68
"#;

        let docs = parse_documents(source).unwrap();

        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name.value, "palette");
        assert!(docs[0].body.is_none());
    }

    #[test]
    fn test_parse_document_without_legend() {
        let source = r#"---
name: simple
---

```px
test
```
"#;

        let docs = parse_documents(source).unwrap();

        assert_eq!(docs.len(), 1);
        assert!(docs[0].legend.is_none());
    }

    #[test]
    fn test_parse_empty_file() {
        let source = "";

        let result = parse_documents(source);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_name() {
        let source = r#"---
tags: #something
---

```px
test
```
"#;

        let result = parse_documents(source);

        assert!(result.is_err());
    }

    #[test]
    fn test_document_spans() {
        let source = r#"---
name: test
---

```px
AB
```
"#;

        let docs = parse_documents(source).unwrap();

        // Document should have valid span
        assert!(docs[0].span.start.line == 1);

        // Body should have span with valid offset
        // Note: line numbers are relative to each section, not absolute
        let body = docs[0].body.as_ref().unwrap();
        assert!(body.span.start.offset > 0);
    }

    #[test]
    fn test_parse_real_example() {
        // Based on platforms.shape.md
        let source = r#"---
name: platform-left
tags: #platform #solid
---

```px
+--
|..
```

---
name: platform-mid
tags: #platform #solid
---

```px
===
...
```

---
=: edge-h

---
name: platform-right
tags: #platform #solid
---

```px
--+
..|
```
"#;

        let docs = parse_documents(source).unwrap();

        assert_eq!(docs.len(), 3);
        assert_eq!(docs[0].name.value, "platform-left");
        assert_eq!(docs[1].name.value, "platform-mid");
        assert_eq!(docs[2].name.value, "platform-right");

        // platform-mid has a legend, others don't
        assert!(docs[0].legend.is_none());
        assert!(docs[1].legend.is_some());
        assert!(docs[2].legend.is_none());
    }

    #[test]
    fn test_get_tags() {
        // Tags with # must be quoted in YAML (otherwise # starts a comment)
        let source = "---\nname: test\ntags: \"#player #solid\"\n---\n";

        let docs = parse_documents(source).unwrap();
        let tags = docs[0].get_tags();

        assert_eq!(tags, vec!["player", "solid"]);
    }
}
