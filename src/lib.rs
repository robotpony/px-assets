//! px - Sprite and map pipeline generator
//!
//! A library for transforming markdown-style definition files into sprites
//! and sprite maps for various platforms.

pub mod cli;
pub mod error;
pub mod parser;
pub mod render;
pub mod types;

pub use error::{PxError, Result};
pub use render::{scale_pixels, write_png, RenderedShape, ShapeRenderer};
pub use types::{
    Brush, BuiltinBrushes, BuiltinShaders, BuiltinStamps, Colour, ColourExpr, Effect, EffectParam,
    ExprEvaluator, LegendEntry, Palette, PixelToken, Shader, ShaderBuilder, Shape, Stamp,
};
