//! Core domain types for px.
//!
//! This module contains the fundamental types used throughout the pipeline:
//! - `Colour` - RGBA colour values
//! - `Palette` - Named colour collections with variants
//! - `ColourExpr` - Colour expressions (darken, lighten, mix, etc.)
//! - `Stamp` - Pixel art patterns with semantic tokens
//! - `PixelToken` - Semantic pixel values (edge, fill, transparent)
//! - `Brush` - Tiling patterns with positional colour tokens
//! - `Shader` - Palette binding and post-processing effects
//! - `Shape` - ASCII compositions that map to stamps/brushes

mod brush;
mod colour;
mod expr;
mod palette;
mod shader;
mod shape;
mod stamp;

pub use brush::{Brush, BuiltinBrushes};
pub use colour::Colour;
pub use expr::{ColourExpr, ExprEvaluator};
pub use palette::{Palette, PaletteBuilder};
pub use shader::{BuiltinShaders, Effect, EffectParam, Shader, ShaderBuilder};
pub use shape::{LegendEntry, Shape};
pub use stamp::{BuiltinStamps, PixelToken, Stamp};
