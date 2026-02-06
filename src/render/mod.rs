//! Rendering module for px.
//!
//! This module handles converting shapes to pixel grids using stamps,
//! palettes, and shaders.

mod png;
mod prefab;
mod shape;

pub use png::{scale_pixels, write_png};
pub use prefab::PrefabRenderer;
pub use shape::{RenderedShape, ShapeRenderer};
