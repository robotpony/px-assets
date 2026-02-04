//! Core domain types for px.
//!
//! This module contains the fundamental types used throughout the pipeline:
//! - `Colour` - RGBA colour values
//! - `Palette` - Named colour collections with variants
//! - `ColourExpr` - Colour expressions (darken, lighten, mix, etc.)
//! - `Stamp` - Pixel art patterns with semantic tokens
//! - `PixelToken` - Semantic pixel values (edge, fill, transparent)

mod colour;
mod expr;
mod palette;
mod stamp;

pub use colour::Colour;
pub use expr::{ColourExpr, ExprEvaluator};
pub use palette::{Palette, PaletteBuilder};
pub use stamp::{BuiltinStamps, PixelToken, Stamp};
