//! Asset type definitions for the registry.
//!
//! Assets are identified by their kind (type) and name, allowing
//! different asset types to share names (e.g., "wall" shape and "wall" brush).

use std::fmt;

/// The kind of asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Palette,
    Stamp,
    Brush,
    Shader,
    Shape,
    Prefab,
    Map,
}

impl AssetKind {
    /// Get the file extension for this asset kind.
    pub fn extension(&self) -> &'static str {
        match self {
            AssetKind::Palette => "palette.md",
            AssetKind::Stamp => "stamp.md",
            AssetKind::Brush => "brush.md",
            AssetKind::Shader => "shader.md",
            AssetKind::Shape => "shape.md",
            AssetKind::Prefab => "prefab.md",
            AssetKind::Map => "map.md",
        }
    }

    /// Get the short name for this kind.
    pub fn name(&self) -> &'static str {
        match self {
            AssetKind::Palette => "palette",
            AssetKind::Stamp => "stamp",
            AssetKind::Brush => "brush",
            AssetKind::Shader => "shader",
            AssetKind::Shape => "shape",
            AssetKind::Prefab => "prefab",
            AssetKind::Map => "map",
        }
    }
}

impl fmt::Display for AssetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A unique identifier for an asset.
///
/// Combines the asset kind and name, allowing different types
/// to have the same name (e.g., shape:wall and brush:wall).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId {
    pub kind: AssetKind,
    pub name: String,
}

impl AssetId {
    /// Create a new asset ID.
    pub fn new(kind: AssetKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }

    /// Create a palette asset ID.
    pub fn palette(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Palette, name)
    }

    /// Create a stamp asset ID.
    pub fn stamp(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Stamp, name)
    }

    /// Create a brush asset ID.
    pub fn brush(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Brush, name)
    }

    /// Create a shader asset ID.
    pub fn shader(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Shader, name)
    }

    /// Create a shape asset ID.
    pub fn shape(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Shape, name)
    }

    /// Create a prefab asset ID.
    pub fn prefab(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Prefab, name)
    }

    /// Create a map asset ID.
    pub fn map(name: impl Into<String>) -> Self {
        Self::new(AssetKind::Map, name)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind, self.name)
    }
}

/// A reference to an asset, used in dependency tracking.
///
/// This is similar to AssetId but used specifically for
/// expressing dependencies between assets.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetRef {
    /// The kind of asset being referenced, if known.
    /// None means the reference could be any type (resolved later).
    pub kind: Option<AssetKind>,
    /// The name of the referenced asset.
    pub name: String,
}

impl AssetRef {
    /// Create a new asset reference with known kind.
    pub fn new(kind: AssetKind, name: impl Into<String>) -> Self {
        Self {
            kind: Some(kind),
            name: name.into(),
        }
    }

    /// Create an untyped reference (kind resolved at lookup time).
    pub fn untyped(name: impl Into<String>) -> Self {
        Self {
            kind: None,
            name: name.into(),
        }
    }

    /// Try to resolve this reference to an AssetId.
    ///
    /// Returns None if the kind is unknown.
    pub fn to_id(&self) -> Option<AssetId> {
        self.kind.map(|k| AssetId::new(k, &self.name))
    }
}

impl fmt::Display for AssetRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Some(kind) => write!(f, "{}:{}", kind, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id_display() {
        let id = AssetId::shape("wall");
        assert_eq!(id.to_string(), "shape:wall");

        let id = AssetId::palette("dungeon");
        assert_eq!(id.to_string(), "palette:dungeon");
    }

    #[test]
    fn test_asset_id_equality() {
        let a = AssetId::shape("wall");
        let b = AssetId::shape("wall");
        let c = AssetId::brush("wall");

        assert_eq!(a, b);
        assert_ne!(a, c); // Same name, different kind
    }

    #[test]
    fn test_asset_ref_to_id() {
        let typed = AssetRef::new(AssetKind::Shape, "wall");
        assert_eq!(typed.to_id(), Some(AssetId::shape("wall")));

        let untyped = AssetRef::untyped("wall");
        assert_eq!(untyped.to_id(), None);
    }

    #[test]
    fn test_asset_kind_extension() {
        assert_eq!(AssetKind::Shape.extension(), "shape.md");
        assert_eq!(AssetKind::Palette.extension(), "palette.md");
    }
}
