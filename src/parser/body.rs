//! Code block body extraction.

use super::span::{Span, Spanned};

/// Result of extracting a code block body.
#[derive(Debug)]
pub struct BodyResult {
    /// The content inside the code block (without fence markers)
    pub content: Spanned<String>,
    /// Byte offset where content after the code block begins
    pub content_end: usize,
}

/// Extract the body content from a ```px code block.
///
/// Searches for ```px (or ```px with optional language hint) and extracts
/// the content until the closing ```.
pub fn extract_body(source: &str, base_offset: usize) -> Option<BodyResult> {
    // Find opening fence
    let fence_start = find_code_fence_start(source)?;
    let after_fence = &source[fence_start..];

    // Find the end of the opening fence line
    let fence_line_end = after_fence.find('\n').unwrap_or(after_fence.len());
    let content_start = fence_start + fence_line_end + 1;

    if content_start > source.len() {
        return None;
    }

    // Find closing fence
    let content_section = &source[content_start..];
    let closing_offset = find_closing_fence(content_section)?;

    let body_content = &content_section[..closing_offset];

    // Trim trailing newline if present
    let body_trimmed = body_content.strip_suffix('\n').unwrap_or(body_content);

    let span = Span::from_local_offsets(
        source,
        content_start,
        content_start + body_trimmed.len(),
        base_offset,
    );

    // Calculate end position (after closing ``` and newline)
    let after_closing = &content_section[closing_offset..];
    let fence_end = after_closing.find('\n').map(|i| i + 1).unwrap_or(3);
    let content_end = content_start + closing_offset + fence_end;

    Some(BodyResult {
        content: Spanned::new(body_trimmed.to_string(), span),
        content_end,
    })
}

/// Find the start of a ```px code fence.
fn find_code_fence_start(source: &str) -> Option<usize> {
    let mut offset = 0;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```px") || trimmed.starts_with("``` px") {
            // Return offset to the start of ```
            let leading = line.len() - trimmed.len();
            return Some(offset + leading);
        }
        offset += line.len() + 1; // +1 for newline
    }
    None
}

/// Find the closing ``` fence.
fn find_closing_fence(source: &str) -> Option<usize> {
    let mut offset = 0;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "```" {
            return Some(offset);
        }
        offset += line.len() + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_body() {
        let source = "```px\nhello\nworld\n```\n";

        let result = extract_body(source, 0).unwrap();

        assert_eq!(result.content.value, "hello\nworld");
    }

    #[test]
    fn test_extract_body_with_prefix() {
        let source = "some text\n```px\nbody\n```\nafter";

        let result = extract_body(source, 0).unwrap();

        assert_eq!(result.content.value, "body");
    }

    #[test]
    fn test_extract_body_no_fence() {
        let source = "no code fence here";

        let result = extract_body(source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_body_unclosed() {
        let source = "```px\nhello\nworld";

        let result = extract_body(source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_body_span_location() {
        let source = "```px\nAB\nCD\n```";

        let result = extract_body(source, 0).unwrap();

        // Content starts at line 2
        assert_eq!(result.content.span.start.line, 2);
        assert_eq!(result.content.span.start.column, 1);
    }

    #[test]
    fn test_extract_body_with_base_offset() {
        let source = "```px\ntest\n```";

        let result = extract_body(source, 100).unwrap();

        assert!(result.content.span.start.offset >= 100);
    }

    #[test]
    fn test_extract_multiline_shape() {
        let source = "```px\n+--+\n|..|\n+--+\n```";

        let result = extract_body(source, 0).unwrap();

        assert_eq!(result.content.value, "+--+\n|..|\n+--+");
    }
}
