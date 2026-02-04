//! Parser modules for px definition files.
//!
//! This module provides infrastructure for parsing markdown-style definition
//! files used by px. Each file can contain one or more document definitions.
//!
//! # Document Structure
//!
//! Each document has:
//! - YAML frontmatter between `---` markers (must include `name:`)
//! - Optional body content inside a px code fence
//! - Optional legend section with glyph mappings
//!
//! # Usage
//!
//! ```ignore
//! use px::parser::parse_documents;
//!
//! let source = std::fs::read_to_string("shapes/wall.shape.md")?;
//! let documents = parse_documents(&source)?;
//!
//! for doc in documents {
//!     println!("Found: {}", doc.name.value);
//! }
//! ```

mod body;
mod document;
mod frontmatter;
mod legend;
pub mod brush;
pub mod palette;
pub mod span;
pub mod stamp;
pub mod types;

// Re-export main entry points
pub use brush::parse_brush_file;
pub use document::parse_documents;
pub use palette::parse_palette;
pub use span::{Location, Span, Spanned};
pub use stamp::parse_stamp_file;
pub use types::{LegendValue, RawDocument};

// Future parser modules for specific file types:
// pub mod shader;
// pub mod shape;
// pub mod prefab;
// pub mod map;
// pub mod target;
