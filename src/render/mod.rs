//! Rendering module for px.
//!
//! This module handles converting shapes to pixel grids using stamps,
//! palettes, and shaders.

mod map;
mod p8;
mod png;
mod prefab;
mod shape;
mod sheet;

pub use map::MapRenderer;
pub use p8::{sprites_that_fit, write_p8, DitherMethod, P8Config};
pub use png::{scale_pixels, write_png};
pub use prefab::PrefabRenderer;
pub use shape::{RenderedShape, ShapeRenderer};
pub use sheet::{write_sheet_json, SheetMeta, SheetPacker};
