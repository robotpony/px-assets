//! Diagnostic types for validation results.

use std::fmt;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A single validation diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level.
    pub severity: Severity,
    /// Machine-readable diagnostic code (e.g. "px::validate::missing-stamp").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional help text suggesting how to fix the issue.
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            help: None,
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            help: None,
        }
    }

    /// Add help text to this diagnostic.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Collects diagnostics from validation checks.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationResult {
    /// Create an empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add an error diagnostic.
    pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.push(Diagnostic::error(code, message));
    }

    /// Add a warning diagnostic.
    pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.push(Diagnostic::warning(code, message));
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning)
    }

    /// Count errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Count warnings.
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Check if there are no diagnostics at all.
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Iterate over diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_result() {
        let result = ValidationResult::new();
        assert!(result.is_ok());
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_error_diagnostic() {
        let mut result = ValidationResult::new();
        result.error("px::test", "something broke");

        assert!(result.has_errors());
        assert!(!result.has_warnings());
        assert!(!result.is_ok());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_warning_diagnostic() {
        let mut result = ValidationResult::new();
        result.warning("px::test", "something looks off");

        assert!(!result.has_errors());
        assert!(result.has_warnings());
        assert!(!result.is_ok());
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn test_merge() {
        let mut a = ValidationResult::new();
        a.error("px::a", "error a");

        let mut b = ValidationResult::new();
        b.warning("px::b", "warning b");

        a.merge(b);
        assert_eq!(a.error_count(), 1);
        assert_eq!(a.warning_count(), 1);
    }

    #[test]
    fn test_diagnostic_with_help() {
        let d = Diagnostic::error("px::test", "missing stamp")
            .with_help("Define the stamp in a .stamp.md file");
        assert_eq!(d.help.as_deref(), Some("Define the stamp in a .stamp.md file"));
    }
}
