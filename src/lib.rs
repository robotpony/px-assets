//! px - Sprite and map pipeline generator
//!
//! A library for transforming markdown-style definition files into sprites
//! and sprite maps for various platforms.

pub mod cli;
pub mod discovery;
pub mod error;
pub mod parser;
pub mod registry;
pub mod render;
pub mod types;
pub mod validation;

pub use discovery::{discover, discover_paths, DiscoveryResult, LoadOptions, Manifest, ScanResult};
pub use error::{PxError, Result};
pub use registry::{AssetId, AssetKind, AssetRegistry, RegistryBuilder};
pub use render::{scale_pixels, write_png, MapRenderer, PrefabRenderer, RenderedShape, ShapeRenderer};
pub use types::{
    Brush, BuiltinBrushes, BuiltinShaders, BuiltinStamps, Colour, ColourExpr, Effect, EffectParam,
    ExprEvaluator, LegendEntry, Map, MapInstance, MapMetadata, Palette, PixelToken, Prefab, Shader,
    ShaderBuilder, Shape, Stamp,
};
pub use validation::{validate_registry, Diagnostic, Severity, ValidationResult};
