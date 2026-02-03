//! YAML frontmatter extraction.

use std::collections::HashMap;

use crate::error::{PxError, Result};

use super::span::{Span, Spanned};

/// Result of extracting frontmatter from a document section.
#[derive(Debug)]
pub struct FrontmatterResult {
    /// Parsed frontmatter key-value pairs
    pub values: HashMap<String, Spanned<serde_yaml::Value>>,
    /// Span of the frontmatter section (including --- markers)
    pub span: Span,
    /// Byte offset where content after frontmatter begins
    pub content_start: usize,
}

/// Extract YAML frontmatter from the beginning of a document.
///
/// Expects the document to start with `---`, contain YAML, and end with `---`.
/// Returns the parsed values and the position where remaining content begins.
pub fn extract_frontmatter(source: &str, base_offset: usize) -> Result<FrontmatterResult> {
    let trimmed = source.trim_start();
    let leading_whitespace = source.len() - trimmed.len();

    // Must start with ---
    if !trimmed.starts_with("---") {
        return Err(PxError::Parse {
            message: "Document must start with ---".to_string(),
            help: Some("Add YAML frontmatter: ---\\nname: my-name\\n---".to_string()),
        });
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    let first_newline = after_first.find('\n').unwrap_or(after_first.len());
    let yaml_start = 3 + first_newline + 1; // Skip past first --- and newline

    // Find closing ---
    let yaml_section = &trimmed[yaml_start..];
    let closing = find_closing_delimiter(yaml_section);

    let (yaml_content, closing_offset) = match closing {
        Some(offset) => (&yaml_section[..offset], yaml_start + offset),
        None => {
            return Err(PxError::Parse {
                message: "Unclosed frontmatter: missing closing ---".to_string(),
                help: Some("Add --- after the YAML content".to_string()),
            });
        }
    };

    // Parse YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_content).map_err(|e| {
        PxError::Parse {
            message: format!("Invalid YAML in frontmatter: {}", e),
            help: None,
        }
    })?;

    // Convert to HashMap with spans
    let values = match parsed {
        serde_yaml::Value::Mapping(map) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                if let Some(key_str) = key.as_str() {
                    // For now, use the frontmatter span for all values
                    // More precise spans would require a custom YAML parser
                    let value_span = Span::from_local_offsets(
                        source,
                        leading_whitespace + yaml_start,
                        leading_whitespace + closing_offset,
                        base_offset,
                    );
                    result.insert(key_str.to_string(), Spanned::new(value, value_span));
                }
            }
            result
        }
        serde_yaml::Value::Null => HashMap::new(),
        _ => {
            return Err(PxError::Parse {
                message: "Frontmatter must be a YAML mapping".to_string(),
                help: Some("Use key: value format".to_string()),
            });
        }
    };

    // Calculate where content starts (after closing --- and newline)
    let after_closing = &trimmed[closing_offset + 3..];
    let newline_after = after_closing.find('\n').map(|i| i + 1).unwrap_or(0);
    let content_start = leading_whitespace + closing_offset + 3 + newline_after;

    let span = Span::from_local_offsets(
        source,
        leading_whitespace,
        leading_whitespace + closing_offset + 3,
        base_offset,
    );

    Ok(FrontmatterResult {
        values,
        span,
        content_start,
    })
}

/// Find the closing --- delimiter in a string.
///
/// The delimiter must be at the start of a line.
fn find_closing_delimiter(s: &str) -> Option<usize> {
    let mut offset = 0;
    for line in s.lines() {
        if line.trim() == "---" {
            return Some(offset);
        }
        offset += line.len() + 1; // +1 for newline
    }
    // Check if the last line (without trailing newline) is ---
    if s.ends_with("---") && !s.ends_with("\n---") {
        Some(s.len() - 3)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_frontmatter() {
        let source = "---\nname: test\ntags: #player\n---\nbody content";

        let result = extract_frontmatter(source, 0).unwrap();

        assert!(result.values.contains_key("name"));
        assert_eq!(
            result.values.get("name").unwrap().value.as_str(),
            Some("test")
        );
        assert!(result.values.contains_key("tags"));
        assert_eq!(result.content_start, 33); // After closing "---\n"
    }

    #[test]
    fn test_extract_frontmatter_no_opening() {
        let source = "name: test\n---\nbody";

        let result = extract_frontmatter(source, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_frontmatter_unclosed() {
        let source = "---\nname: test\nbody content";

        let result = extract_frontmatter(source, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_empty_frontmatter() {
        let source = "---\n---\nbody";

        let result = extract_frontmatter(source, 0).unwrap();
        assert!(result.values.is_empty());
    }

    #[test]
    fn test_extract_frontmatter_with_base_offset() {
        let source = "---\nname: test\n---\n";

        let result = extract_frontmatter(source, 100).unwrap();

        // Spans should include base offset
        assert!(result.span.start.offset >= 100);
    }
}
