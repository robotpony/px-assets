//! Legend section parsing.

use std::collections::HashMap;

use crate::error::{PxError, Result};

use super::span::{Span, Spanned};
use super::types::LegendValue;

/// Result of extracting a legend section.
#[derive(Debug)]
#[allow(dead_code)] // Fields will be used in future phases
pub struct LegendResult {
    /// Parsed legend mappings
    pub entries: HashMap<char, Spanned<LegendValue>>,
    /// Span of the legend section
    pub span: Span,
    /// Byte offset where content after legend begins
    pub content_end: usize,
}

/// Extract legend section from after a code block.
///
/// Legend starts with `---` and contains character mappings.
/// It ends at EOF or the next `---` that starts a new definition (has `name:`).
pub fn extract_legend(source: &str, base_offset: usize) -> Result<Option<LegendResult>> {
    // Skip leading whitespace and HTML comments before the --- delimiter
    let trimmed = skip_html_comments(source.trim_start());
    let leading_whitespace = source.len() - trimmed.len();

    // Legend must start with ---
    if !trimmed.starts_with("---") {
        return Ok(None);
    }

    // Skip the --- line
    let after_delimiter = &trimmed[3..];
    let first_newline = after_delimiter.find('\n').unwrap_or(after_delimiter.len());
    let legend_start = 3 + first_newline + 1;

    if legend_start > trimmed.len() {
        return Ok(None);
    }

    // Find where legend ends (next --- with name: or EOF)
    let legend_section = &trimmed[legend_start..];
    let legend_end = find_legend_end(legend_section);
    let legend_content = &legend_section[..legend_end];

    if legend_content.trim().is_empty() {
        return Ok(None);
    }

    // Parse legend entries
    let mut entries = HashMap::new();
    let mut line_offset = legend_start;

    for line in legend_content.lines() {
        let line_trimmed = line.trim();

        if !line_trimmed.is_empty() && line_trimmed.contains(':') {
            let (glyph, value) = parse_legend_line(line_trimmed)?;

            let value_span = Span::from_local_offsets(
                source,
                leading_whitespace + line_offset,
                leading_whitespace + line_offset + line.len(),
                base_offset,
            );

            entries.insert(glyph, Spanned::new(value, value_span));
        }

        line_offset += line.len() + 1;
    }

    let span = Span::from_local_offsets(
        source,
        leading_whitespace,
        leading_whitespace + legend_start + legend_end,
        base_offset,
    );

    Ok(Some(LegendResult {
        entries,
        span,
        content_end: leading_whitespace + legend_start + legend_end,
    }))
}

/// Skip leading HTML comment lines and whitespace.
///
/// Markdown files may contain HTML comments between the code block
/// and the legend `---` delimiter. Strip them so the delimiter is found.
fn skip_html_comments(s: &str) -> &str {
    let mut remaining = s;
    loop {
        let trimmed = remaining.trim_start();
        if trimmed.starts_with("<!--") {
            // Find end of comment
            if let Some(end) = trimmed.find("-->") {
                remaining = trimmed[end + 3..].trim_start();
                continue;
            }
        }
        return trimmed;
    }
}

/// Find where the legend section ends.
///
/// Legend ends at:
/// - A line starting with `---` followed by a line containing `name:`
/// - End of string
fn find_legend_end(s: &str) -> usize {
    let lines: Vec<&str> = s.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "---" {
            // Check if next line has name: (new definition)
            if let Some(next_line) = lines.get(i + 1) {
                if next_line.trim().starts_with("name:") {
                    // Calculate offset up to this ---
                    return lines[..i].iter().map(|l| l.len() + 1).sum();
                }
            }
        }
    }

    s.len()
}

/// Parse a single legend line into (glyph, value).
fn parse_legend_line(line: &str) -> Result<(char, LegendValue)> {
    // Handle quoted characters: "x": value or 'x': value
    let (glyph, rest) = if line.starts_with('"') || line.starts_with('\'') {
        let quote = line.chars().next().unwrap();
        let end_quote = line[1..].find(quote).ok_or_else(|| PxError::Parse {
            message: format!("Unclosed quote in legend: {}", line),
            help: None,
        })?;
        let glyph_str = &line[1..=end_quote];
        if glyph_str.len() != 1 {
            return Err(PxError::Parse {
                message: format!("Legend glyph must be a single character: {}", glyph_str),
                help: None,
            });
        }
        let glyph = glyph_str.chars().next().unwrap();
        let rest = line[end_quote + 2..].trim_start();
        // Skip the colon
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
        (glyph, rest)
    } else {
        // Unquoted: x: value
        let colon_pos = line.find(':').ok_or_else(|| PxError::Parse {
            message: format!("Legend line must contain ':': {}", line),
            help: None,
        })?;
        let glyph_str = line[..colon_pos].trim();
        if glyph_str.len() != 1 {
            return Err(PxError::Parse {
                message: format!("Legend glyph must be a single character: {}", glyph_str),
                help: Some("Use quotes for special characters: \" \": empty".to_string()),
            });
        }
        let glyph = glyph_str.chars().next().unwrap();
        let rest = line[colon_pos + 1..].trim();
        (glyph, rest)
    };

    // Parse the value
    let value = parse_legend_value(rest)?;

    Ok((glyph, value))
}

/// Parse a legend value (simple reference or complex object).
fn parse_legend_value(s: &str) -> Result<LegendValue> {
    let trimmed = s.trim();

    // Check for complex value: { ... }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        parse_complex_legend_value(trimmed)
    } else {
        // Simple reference
        Ok(LegendValue::Reference(trimmed.to_string()))
    }
}

/// Parse a complex legend value: { fill: checker, A: $edge, B: $fill }
fn parse_complex_legend_value(s: &str) -> Result<LegendValue> {
    let inner = &s[1..s.len() - 1]; // Strip { }

    let mut name = None;
    let mut fill = false;
    let mut bindings = HashMap::new();

    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let colon_pos = part.find(':').ok_or_else(|| PxError::Parse {
            message: format!("Invalid legend entry: {}", part),
            help: None,
        })?;

        let key = part[..colon_pos].trim();
        let value = part[colon_pos + 1..].trim();

        match key {
            "stamp" => {
                name = Some(value.to_string());
                fill = false;
            }
            "fill" => {
                name = Some(value.to_string());
                fill = true;
            }
            _ if key.len() == 1 => {
                // Single char key = colour binding
                let binding_char = key.chars().next().unwrap();
                bindings.insert(binding_char, value.to_string());
            }
            _ => {
                return Err(PxError::Parse {
                    message: format!("Unknown legend key: {}", key),
                    help: None,
                });
            }
        }
    }

    let name = name.ok_or_else(|| PxError::Parse {
        message: "Legend entry missing 'stamp' or 'fill' key".to_string(),
        help: None,
    })?;

    Ok(LegendValue::Complex {
        name,
        fill,
        bindings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_legend() {
        let source = "---\nB: brick\nW: wall\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        assert_eq!(result.entries.len(), 2);
        assert!(matches!(
            &result.entries.get(&'B').unwrap().value,
            LegendValue::Reference(s) if s == "brick"
        ));
    }

    #[test]
    fn test_extract_legend_with_quoted_char() {
        let source = "---\n\" \": empty\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        assert!(matches!(
            &result.entries.get(&' ').unwrap().value,
            LegendValue::Reference(s) if s == "empty"
        ));
    }

    #[test]
    fn test_extract_complex_legend_entry() {
        let source = "---\n~: { fill: checker, A: $edge, B: $fill }\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        match &result.entries.get(&'~').unwrap().value {
            LegendValue::Complex { name, fill, bindings } => {
                assert_eq!(name, "checker");
                assert!(fill);
                assert_eq!(bindings.get(&'A'), Some(&"$edge".to_string()));
                assert_eq!(bindings.get(&'B'), Some(&"$fill".to_string()));
            }
            _ => panic!("Expected complex legend value"),
        }
    }

    #[test]
    fn test_extract_legend_stamp_not_fill() {
        let source = "---\nC: { stamp: checker, A: $edge }\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        match &result.entries.get(&'C').unwrap().value {
            LegendValue::Complex { fill, .. } => {
                assert!(!fill);
            }
            _ => panic!("Expected complex legend value"),
        }
    }

    #[test]
    fn test_legend_ends_at_new_definition() {
        let source = "---\nB: brick\n\n---\nname: next-shape\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        // Should only have B, not parse into next definition
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_no_legend() {
        let source = "just some text without delimiter";

        let result = extract_legend(source, 0).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_legend_with_html_comments() {
        let source = "<!-- comment -->\n<!-- another -->\n---\nB: brick\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        assert_eq!(result.entries.len(), 1);
        assert!(matches!(
            &result.entries.get(&'B').unwrap().value,
            LegendValue::Reference(s) if s == "brick"
        ));
    }

    #[test]
    fn test_extract_legend_with_multiline_html_comment() {
        let source = "<!-- multi\nline\ncomment -->\n---\nW: wall\n";

        let result = extract_legend(source, 0).unwrap().unwrap();

        assert_eq!(result.entries.len(), 1);
        assert!(matches!(
            &result.entries.get(&'W').unwrap().value,
            LegendValue::Reference(s) if s == "wall"
        ));
    }

    #[test]
    fn test_empty_legend() {
        let source = "---\n\n---\nname: next\n";

        let result = extract_legend(source, 0).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_legend_line_unclosed_quote() {
        let result = parse_legend_line("\"x: brick");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_legend_value_empty_braces() {
        let result = parse_complex_legend_value("{}");

        // Empty braces have no stamp/fill key → error
        assert!(result.is_err());
    }
}
