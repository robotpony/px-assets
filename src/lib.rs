//! px - Sprite and map pipeline generator
//!
//! A library for transforming markdown-style definition files into sprites
//! and sprite maps for various platforms.

pub mod cli;
pub mod error;
pub mod parser;
pub mod types;

pub use error::{PxError, Result};
pub use types::{Colour, ColourExpr, ExprEvaluator, Palette};
