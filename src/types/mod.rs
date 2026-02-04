//! Core domain types for px.
//!
//! This module contains the fundamental types used throughout the pipeline:
//! - `Colour` - RGBA colour values
//! - `Palette` - Named colour collections with variants
//! - `ColourExpr` - Colour expressions (darken, lighten, mix, etc.)

mod colour;
mod expr;
mod palette;

pub use colour::Colour;
pub use expr::{ColourExpr, ExprEvaluator};
pub use palette::{Palette, PaletteBuilder};
