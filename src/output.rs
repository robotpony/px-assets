//! Terminal output formatting for px CLI.
//!
//! Provides Cargo-style status output with right-aligned coloured verbs.
//! All status output goes to stderr; stdout is reserved for machine-readable output.

use std::io::{self, IsTerminal, Write};

/// ANSI escape codes.
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

/// Width for right-aligned verb column.
const VERB_WIDTH: usize = 12;

/// Terminal-aware status printer.
///
/// Prints Cargo-style status lines to stderr with optional ANSI colours.
/// Colour is enabled when stderr is a terminal.
pub struct Printer {
    color: bool,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            color: io::stderr().is_terminal(),
        }
    }

    /// Print a status line with a green bold verb.
    /// e.g. "   Compiling player-stand (8x12)"
    pub fn status(&self, verb: &str, message: &str) {
        self.print_line(GREEN, verb, message);
    }

    /// Print a success/completion line with a green bold verb.
    pub fn success(&self, verb: &str, message: &str) {
        self.print_line(GREEN, verb, message);
    }

    /// Print an informational line with a cyan bold verb.
    pub fn info(&self, verb: &str, message: &str) {
        self.print_line(CYAN, verb, message);
    }

    /// Print a warning line with a yellow bold verb.
    pub fn warning(&self, verb: &str, message: &str) {
        self.print_line(YELLOW, verb, message);
    }

    /// Print an error line with a red bold verb.
    pub fn error(&self, verb: &str, message: &str) {
        self.print_line(RED, verb, message);
    }

    /// Format a string as dim/grey.
    pub fn dim(&self, text: &str) -> String {
        if self.color {
            format!("{DIM}{text}{RESET}")
        } else {
            text.to_string()
        }
    }

    /// Format a string as bold.
    pub fn bold(&self, text: &str) -> String {
        if self.color {
            format!("{BOLD}{text}{RESET}")
        } else {
            text.to_string()
        }
    }

    /// Format a string as cyan (for paths, info).
    pub fn cyan(&self, text: &str) -> String {
        if self.color {
            format!("{CYAN}{text}{RESET}")
        } else {
            text.to_string()
        }
    }

    /// Format a diagnostic severity label with colour.
    pub fn severity(&self, label: &str, is_error: bool) -> String {
        let color = if is_error { RED } else { YELLOW };
        if self.color {
            format!("{BOLD}{color}{label}{RESET}")
        } else {
            label.to_string()
        }
    }

    fn print_line(&self, color: &str, verb: &str, message: &str) {
        let mut stderr = io::stderr().lock();
        if self.color {
            let _ = writeln!(
                stderr,
                "{BOLD}{color}{verb:>VERB_WIDTH$}{RESET} {message}"
            );
        } else {
            let _ = writeln!(stderr, "{verb:>VERB_WIDTH$} {message}");
        }
    }
}

/// Pluralize a count: `plural(1, "map", "maps")` → "1 map".
pub fn plural(n: usize, singular: &str, pluralized: &str) -> String {
    if n == 1 {
        format!("{} {}", n, singular)
    } else {
        format!("{} {}", n, pluralized)
    }
}

/// Return a relative display path when possible, absolute otherwise.
pub fn display_path(path: &std::path::Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&cwd) {
            let s = relative.display().to_string();
            if s.is_empty() {
                return ".".to_string();
            }
            return s;
        }
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plural_singular() {
        assert_eq!(plural(1, "map", "maps"), "1 map");
    }

    #[test]
    fn test_plural_zero() {
        assert_eq!(plural(0, "map", "maps"), "0 maps");
    }

    #[test]
    fn test_plural_many() {
        assert_eq!(plural(5, "shape", "shapes"), "5 shapes");
    }

    #[test]
    fn test_display_path_absolute() {
        use std::path::Path;
        // An absolute path outside cwd should stay absolute
        let p = Path::new("/nonexistent/path/to/file");
        assert_eq!(display_path(p), "/nonexistent/path/to/file");
    }
}
