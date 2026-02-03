//! Source location tracking for error messages.

use std::fmt;

/// A location in source text (byte offset, line, column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Location {
    /// Byte offset from start of file
    pub offset: usize,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed, in characters not bytes)
    pub column: u32,
}

impl Location {
    pub fn new(offset: usize, line: u32, column: u32) -> Self {
        Self { offset, line, column }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A span in source text (start and end locations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start location (inclusive)
    pub start: Location,
    /// End location (exclusive)
    pub end: Location,
}

impl Span {
    pub fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }

    /// Create a span from byte offsets, calculating line/column from source.
    ///
    /// The offsets are relative to the source string. Use `from_local_offsets`
    /// if you need to add a base offset for absolute positioning.
    pub fn from_offsets(source: &str, start_offset: usize, end_offset: usize) -> Self {
        Self {
            start: offset_to_location(source, start_offset),
            end: offset_to_location(source, end_offset),
        }
    }

    /// Create a span from local offsets within a source string, adding a base offset.
    ///
    /// The local_start and local_end are relative to the source string.
    /// The base_offset is added to get absolute file positions.
    pub fn from_local_offsets(
        source: &str,
        local_start: usize,
        local_end: usize,
        base_offset: usize,
    ) -> Self {
        let mut start = offset_to_location(source, local_start);
        let mut end = offset_to_location(source, local_end);
        start.offset += base_offset;
        end.offset += base_offset;
        Self { start, end }
    }

    /// Byte length of the span.
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Whether the span is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Merge two spans into one covering both.
    pub fn merge(self, other: Span) -> Span {
        let start = if self.start.offset <= other.start.offset {
            self.start
        } else {
            other.start
        };
        let end = if self.end.offset >= other.end.offset {
            self.end
        } else {
            other.end
        };
        Span { start, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(f, "{}:{}-{}", self.start.line, self.start.column, self.end.column)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// A value with an associated source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            value: &self.value,
            span: self.span,
        }
    }
}

impl<T: Default> Default for Spanned<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            span: Span::default(),
        }
    }
}

/// Convert a byte offset to a Location (line/column).
pub fn offset_to_location(source: &str, offset: usize) -> Location {
    let offset = offset.min(source.len());
    let before = &source[..offset];

    let line = before.bytes().filter(|&b| b == b'\n').count() as u32 + 1;
    let last_newline = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let column = before[last_newline..].chars().count() as u32 + 1;

    Location { offset, line, column }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_location_simple() {
        let source = "hello\nworld";

        assert_eq!(offset_to_location(source, 0), Location::new(0, 1, 1));
        assert_eq!(offset_to_location(source, 5), Location::new(5, 1, 6)); // newline
        assert_eq!(offset_to_location(source, 6), Location::new(6, 2, 1)); // 'w'
        assert_eq!(offset_to_location(source, 11), Location::new(11, 2, 6)); // end
    }

    #[test]
    fn test_offset_to_location_empty() {
        let source = "";
        assert_eq!(offset_to_location(source, 0), Location::new(0, 1, 1));
    }

    #[test]
    fn test_span_merge() {
        let source = "hello world";
        let span1 = Span::from_offsets(source, 0, 5);
        let span2 = Span::from_offsets(source, 6, 11);
        let merged = span1.merge(span2);

        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 11);
    }
}
