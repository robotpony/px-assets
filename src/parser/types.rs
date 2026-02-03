//! Common types for parsed documents.

use std::collections::HashMap;

use super::span::{Span, Spanned};

/// A raw parsed document before type-specific processing.
///
/// This represents the common structure of all px definition files:
/// - YAML frontmatter with metadata
/// - Optional body content (inside ```px blocks)
/// - Optional legend section (glyph mappings)
#[derive(Debug, Clone)]
pub struct RawDocument {
    /// The document name (required, from frontmatter)
    pub name: Spanned<String>,

    /// All frontmatter key-value pairs
    pub frontmatter: HashMap<String, Spanned<serde_yaml::Value>>,

    /// Body content (inside ```px block), if present
    pub body: Option<Spanned<String>>,

    /// Legend mappings (character -> reference), if present
    pub legend: Option<HashMap<char, Spanned<LegendValue>>>,

    /// Span covering the entire document
    pub span: Span,
}

/// A legend entry value.
///
/// Legend entries can be simple references or complex objects with colour bindings.
#[derive(Debug, Clone, PartialEq)]
pub enum LegendValue {
    /// Simple reference: `B: brick`
    Reference(String),

    /// Complex entry with stamp/fill and optional colour bindings
    /// `~: { fill: checker, A: $edge, B: $fill }`
    Complex {
        /// The stamp or brush name
        name: String,
        /// Whether this is a fill (tiled) or stamp (single placement)
        fill: bool,
        /// Colour bindings for brush tokens
        bindings: HashMap<char, String>,
    },
}

impl RawDocument {
    /// Get a frontmatter value by key.
    pub fn get_frontmatter(&self, key: &str) -> Option<&Spanned<serde_yaml::Value>> {
        self.frontmatter.get(key)
    }

    /// Get a frontmatter string value by key.
    pub fn get_frontmatter_str(&self, key: &str) -> Option<&str> {
        self.frontmatter
            .get(key)
            .and_then(|v| v.value.as_str())
    }

    /// Get tags from frontmatter (handles both string and sequence).
    pub fn get_tags(&self) -> Vec<String> {
        match self.frontmatter.get("tags") {
            Some(spanned) => match &spanned.value {
                serde_yaml::Value::String(s) => {
                    // Parse space-separated tags, strip # prefix
                    s.split_whitespace()
                        .map(|t| t.trim_start_matches('#').to_string())
                        .collect()
                }
                serde_yaml::Value::Sequence(seq) => {
                    seq.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.trim_start_matches('#').to_string())
                        .collect()
                }
                _ => vec![],
            },
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::span::Location;

    fn dummy_span() -> Span {
        Span::new(Location::new(0, 1, 1), Location::new(0, 1, 1))
    }

    #[test]
    fn test_get_tags_from_string() {
        let mut frontmatter = HashMap::new();
        frontmatter.insert(
            "tags".to_string(),
            Spanned::new(
                serde_yaml::Value::String("#player #solid".to_string()),
                dummy_span(),
            ),
        );

        let doc = RawDocument {
            name: Spanned::new("test".to_string(), dummy_span()),
            frontmatter,
            body: None,
            legend: None,
            span: dummy_span(),
        };

        assert_eq!(doc.get_tags(), vec!["player", "solid"]);
    }

    #[test]
    fn test_legend_value_variants() {
        let simple = LegendValue::Reference("brick".to_string());
        let complex = LegendValue::Complex {
            name: "checker".to_string(),
            fill: true,
            bindings: [('A', "$edge".to_string())].into_iter().collect(),
        };

        assert!(matches!(simple, LegendValue::Reference(_)));
        assert!(matches!(complex, LegendValue::Complex { fill: true, .. }));
    }
}
