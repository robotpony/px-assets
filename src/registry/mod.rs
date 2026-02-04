//! Asset registry for managing px resources.
//!
//! The registry provides centralized storage for all assets (palettes, stamps,
//! brushes, shaders, shapes, prefabs, maps) and tracks dependencies between them.
//!
//! # Example
//!
//! ```ignore
//! use px::registry::{AssetRegistry, RegistryBuilder};
//!
//! let mut builder = RegistryBuilder::new();
//! builder.add_palette(palette);
//! builder.add_shape(shape);
//!
//! let registry = builder.build()?;
//! let build_order = registry.build_order();
//! ```

mod graph;
pub mod types;

use std::collections::HashMap;

use crate::error::{PxError, Result};
use crate::types::{Brush, Palette, Shader, Shape, Stamp};

pub use graph::{CycleError, DependencyGraph};
pub use types::{AssetId, AssetKind, AssetRef};

/// Centralized storage for all px assets.
///
/// The registry is immutable after construction - use `RegistryBuilder`
/// to create a new registry.
#[derive(Debug)]
pub struct AssetRegistry {
    palettes: HashMap<String, Palette>,
    stamps: HashMap<String, Stamp>,
    brushes: HashMap<String, Brush>,
    shaders: HashMap<String, Shader>,
    shapes: HashMap<String, Shape>,
    // prefabs and maps will be added in Phase 2.3/2.4

    /// Dependency graph for all assets.
    graph: DependencyGraph,

    /// Assets in topologically sorted order (dependencies first).
    build_order: Vec<AssetId>,
}

impl AssetRegistry {
    /// Get a palette by name.
    pub fn get_palette(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(name)
    }

    /// Get a stamp by name.
    pub fn get_stamp(&self, name: &str) -> Option<&Stamp> {
        self.stamps.get(name)
    }

    /// Get a brush by name.
    pub fn get_brush(&self, name: &str) -> Option<&Brush> {
        self.brushes.get(name)
    }

    /// Get a shader by name.
    pub fn get_shader(&self, name: &str) -> Option<&Shader> {
        self.shaders.get(name)
    }

    /// Get a shape by name.
    pub fn get_shape(&self, name: &str) -> Option<&Shape> {
        self.shapes.get(name)
    }

    /// Get all palette names.
    pub fn palette_names(&self) -> impl Iterator<Item = &str> {
        self.palettes.keys().map(|s| s.as_str())
    }

    /// Get all stamp names.
    pub fn stamp_names(&self) -> impl Iterator<Item = &str> {
        self.stamps.keys().map(|s| s.as_str())
    }

    /// Get all brush names.
    pub fn brush_names(&self) -> impl Iterator<Item = &str> {
        self.brushes.keys().map(|s| s.as_str())
    }

    /// Get all shader names.
    pub fn shader_names(&self) -> impl Iterator<Item = &str> {
        self.shaders.keys().map(|s| s.as_str())
    }

    /// Get all shape names.
    pub fn shape_names(&self) -> impl Iterator<Item = &str> {
        self.shapes.keys().map(|s| s.as_str())
    }

    /// Get all palettes.
    pub fn palettes(&self) -> impl Iterator<Item = &Palette> {
        self.palettes.values()
    }

    /// Get all stamps.
    pub fn stamps(&self) -> impl Iterator<Item = &Stamp> {
        self.stamps.values()
    }

    /// Get all brushes.
    pub fn brushes(&self) -> impl Iterator<Item = &Brush> {
        self.brushes.values()
    }

    /// Get all shaders.
    pub fn shaders(&self) -> impl Iterator<Item = &Shader> {
        self.shaders.values()
    }

    /// Get all shapes.
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> {
        self.shapes.values()
    }

    /// Get the dependency graph.
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Get assets in build order (dependencies first).
    pub fn build_order(&self) -> &[AssetId] {
        &self.build_order
    }

    /// Get the total number of assets.
    pub fn len(&self) -> usize {
        self.palettes.len()
            + self.stamps.len()
            + self.brushes.len()
            + self.shaders.len()
            + self.shapes.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Builder for constructing an AssetRegistry.
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    palettes: HashMap<String, Palette>,
    stamps: HashMap<String, Stamp>,
    brushes: HashMap<String, Brush>,
    shaders: HashMap<String, Shader>,
    shapes: HashMap<String, Shape>,
}

impl RegistryBuilder {
    /// Create a new registry builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a palette to the registry.
    pub fn add_palette(&mut self, palette: Palette) -> &mut Self {
        self.palettes.insert(palette.name.clone(), palette);
        self
    }

    /// Add multiple palettes.
    pub fn add_palettes(&mut self, palettes: impl IntoIterator<Item = Palette>) -> &mut Self {
        for palette in palettes {
            self.add_palette(palette);
        }
        self
    }

    /// Add a stamp to the registry.
    pub fn add_stamp(&mut self, stamp: Stamp) -> &mut Self {
        self.stamps.insert(stamp.name.clone(), stamp);
        self
    }

    /// Add multiple stamps.
    pub fn add_stamps(&mut self, stamps: impl IntoIterator<Item = Stamp>) -> &mut Self {
        for stamp in stamps {
            self.add_stamp(stamp);
        }
        self
    }

    /// Add a brush to the registry.
    pub fn add_brush(&mut self, brush: Brush) -> &mut Self {
        self.brushes.insert(brush.name.clone(), brush);
        self
    }

    /// Add multiple brushes.
    pub fn add_brushes(&mut self, brushes: impl IntoIterator<Item = Brush>) -> &mut Self {
        for brush in brushes {
            self.add_brush(brush);
        }
        self
    }

    /// Add a shader to the registry.
    pub fn add_shader(&mut self, shader: Shader) -> &mut Self {
        self.shaders.insert(shader.name.clone(), shader);
        self
    }

    /// Add multiple shaders.
    pub fn add_shaders(&mut self, shaders: impl IntoIterator<Item = Shader>) -> &mut Self {
        for shader in shaders {
            self.add_shader(shader);
        }
        self
    }

    /// Add a shape to the registry.
    pub fn add_shape(&mut self, shape: Shape) -> &mut Self {
        self.shapes.insert(shape.name.clone(), shape);
        self
    }

    /// Add multiple shapes.
    pub fn add_shapes(&mut self, shapes: impl IntoIterator<Item = Shape>) -> &mut Self {
        for shape in shapes {
            self.add_shape(shape);
        }
        self
    }

    /// Build the registry, computing dependencies and build order.
    pub fn build(self) -> Result<AssetRegistry> {
        let mut graph = DependencyGraph::new();

        // Register all assets and their dependencies

        // Palettes (may depend on other palettes via inheritance)
        for palette in self.palettes.values() {
            let id = AssetId::palette(&palette.name);
            graph.register(id.clone());

            // Palette inheritance creates a dependency
            // Note: Palette struct doesn't expose parent, so we skip this for now
            // This would need to be added to the Palette type or tracked separately
        }

        // Stamps (no dependencies on other assets currently)
        for stamp in self.stamps.values() {
            let id = AssetId::stamp(&stamp.name);
            graph.register(id);
        }

        // Brushes (no dependencies on other assets currently)
        for brush in self.brushes.values() {
            let id = AssetId::brush(&brush.name);
            graph.register(id);
        }

        // Shaders depend on palettes
        for shader in self.shaders.values() {
            let id = AssetId::shader(&shader.name);
            graph.register(id.clone());

            // Shader depends on its palette
            let palette_id = AssetId::palette(&shader.palette);
            if self.palettes.contains_key(&shader.palette) {
                graph.add_dependency(id, palette_id);
            }
            // Note: we don't error on missing palettes here - that's validation
        }

        // Shapes depend on stamps and brushes via legend
        for shape in self.shapes.values() {
            let id = AssetId::shape(&shape.name);
            graph.register(id.clone());

            // Extract dependencies from legend
            for entry in shape.legend().values() {
                match entry {
                    crate::types::LegendEntry::StampRef(name) => {
                        // Could be a stamp or builtin
                        if self.stamps.contains_key(name) {
                            graph.add_dependency(id.clone(), AssetId::stamp(name));
                        }
                    }
                    crate::types::LegendEntry::BrushRef { name, .. }
                    | crate::types::LegendEntry::Fill { name, .. } => {
                        if self.brushes.contains_key(name) {
                            graph.add_dependency(id.clone(), AssetId::brush(name));
                        }
                    }
                }
            }
        }

        // Compute build order via topological sort
        let build_order = graph.topological_sort().map_err(|e| PxError::Build {
            message: e.to_string(),
            help: Some("Check for circular references between assets".to_string()),
        })?;

        Ok(AssetRegistry {
            palettes: self.palettes,
            stamps: self.stamps,
            brushes: self.brushes,
            shaders: self.shaders,
            shapes: self.shapes,
            graph,
            build_order,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BuiltinStamps, LegendEntry};

    #[test]
    fn test_empty_registry() {
        let registry = RegistryBuilder::new().build().unwrap();
        assert!(registry.is_empty());
        assert!(registry.build_order().is_empty());
    }

    #[test]
    fn test_add_palette() {
        let palette = Palette::default_palette();
        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);

        let registry = builder.build().unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.get_palette("default").is_some());
    }

    #[test]
    fn test_add_stamps() {
        let stamps = BuiltinStamps::all();
        let mut builder = RegistryBuilder::new();
        builder.add_stamps(stamps);

        let registry = builder.build().unwrap();
        assert!(registry.get_stamp("corner").is_some());
        assert!(registry.get_stamp("fill").is_some());
    }

    #[test]
    fn test_shader_depends_on_palette() {
        let palette = Palette::default_palette();
        let shader = Shader::new("test-shader", "default");

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        builder.add_shader(shader);

        let registry = builder.build().unwrap();

        // Palette should come before shader in build order
        let order = registry.build_order();
        let palette_pos = order
            .iter()
            .position(|id| id == &AssetId::palette("default"))
            .unwrap();
        let shader_pos = order
            .iter()
            .position(|id| id == &AssetId::shader("test-shader"))
            .unwrap();

        assert!(palette_pos < shader_pos);
    }

    #[test]
    fn test_shape_depends_on_stamp() {
        let stamp = Stamp::single("brick", Some('B'), crate::types::PixelToken::Edge);

        let mut legend = std::collections::HashMap::new();
        legend.insert('B', LegendEntry::StampRef("brick".to_string()));

        let shape = Shape::new("wall", vec![], vec![vec!['B']], legend);

        let mut builder = RegistryBuilder::new();
        builder.add_stamp(stamp);
        builder.add_shape(shape);

        let registry = builder.build().unwrap();

        // Stamp should come before shape in build order
        let order = registry.build_order();
        let stamp_pos = order
            .iter()
            .position(|id| id == &AssetId::stamp("brick"))
            .unwrap();
        let shape_pos = order
            .iter()
            .position(|id| id == &AssetId::shape("wall"))
            .unwrap();

        assert!(stamp_pos < shape_pos);
    }

    #[test]
    fn test_build_order_complex() {
        // palette -> shader -> (implicit, shapes use palette via shader)
        // stamp -> shape
        // brush -> shape

        let palette = Palette::default_palette();
        let shader = Shader::new("dark", "default");
        let stamp = Stamp::single("brick", Some('B'), crate::types::PixelToken::Edge);
        let brush = Brush::single("solid", 'A');

        let mut legend = std::collections::HashMap::new();
        legend.insert('B', LegendEntry::StampRef("brick".to_string()));
        legend.insert(
            '~',
            LegendEntry::Fill {
                name: "solid".to_string(),
                bindings: std::collections::HashMap::new(),
            },
        );

        let shape = Shape::new("wall", vec![], vec![vec!['B', '~']], legend);

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        builder.add_shader(shader);
        builder.add_stamp(stamp);
        builder.add_brush(brush);
        builder.add_shape(shape);

        let registry = builder.build().unwrap();

        let order = registry.build_order();
        assert_eq!(order.len(), 5);

        // Shape should be last (depends on stamp and brush)
        let shape_pos = order
            .iter()
            .position(|id| id == &AssetId::shape("wall"))
            .unwrap();
        let stamp_pos = order
            .iter()
            .position(|id| id == &AssetId::stamp("brick"))
            .unwrap();
        let brush_pos = order
            .iter()
            .position(|id| id == &AssetId::brush("solid"))
            .unwrap();

        assert!(stamp_pos < shape_pos);
        assert!(brush_pos < shape_pos);
    }

    #[test]
    fn test_get_all_assets() {
        let palette = Palette::default_palette();
        let stamp = Stamp::single("test", None, crate::types::PixelToken::Edge);

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        builder.add_stamp(stamp);

        let registry = builder.build().unwrap();

        let palette_names: Vec<_> = registry.palette_names().collect();
        assert_eq!(palette_names, vec!["default"]);

        let stamp_names: Vec<_> = registry.stamp_names().collect();
        assert_eq!(stamp_names, vec!["test"]);
    }
}
