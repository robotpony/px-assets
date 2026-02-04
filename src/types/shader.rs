//! Shader type for palette binding and effects.
//!
//! Shaders define how colours are resolved and what post-processing effects
//! are applied during rendering. They bind a palette (and optionally a variant)
//! to the rendering pipeline.
//!
//! # Example
//!
//! ```yaml
//! ---
//! name: dungeon-dark
//! palette: dungeon
//! palette_variant: dark-mode
//! ---
//! effects:
//!   - type: vignette
//!     strength: 0.3
//! ```

use std::collections::HashMap;

/// A shader definition for rendering configuration.
#[derive(Debug, Clone)]
pub struct Shader {
    /// Shader name (unique identifier).
    pub name: String,

    /// Palette to use for colour resolution (required).
    pub palette: String,

    /// Palette variant to activate (optional).
    pub palette_variant: Option<String>,

    /// Post-processing effects to apply.
    pub effects: Vec<Effect>,

    /// Parent shader name for inheritance.
    inherits: Option<String>,
}

impl Shader {
    /// Create a new shader with just a palette binding.
    pub fn new(name: impl Into<String>, palette: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            palette: palette.into(),
            palette_variant: None,
            effects: Vec::new(),
            inherits: None,
        }
    }

    /// Set the palette variant.
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.palette_variant = Some(variant.into());
        self
    }

    /// Add an effect.
    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    /// Set the parent shader for inheritance.
    pub fn with_inherits(mut self, parent: impl Into<String>) -> Self {
        self.inherits = Some(parent.into());
        self
    }

    /// Get the parent shader name if set.
    pub fn parent_name(&self) -> Option<&str> {
        self.inherits.as_deref()
    }

    /// Check if this shader has any effects.
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }

    /// Merge settings from a parent shader.
    ///
    /// Child settings take precedence over parent settings.
    pub fn merge_from(&mut self, parent: &Shader) {
        // Palette is required, so we don't inherit it unless unset
        // (which shouldn't happen in valid shaders)

        // Inherit variant if not set
        if self.palette_variant.is_none() {
            self.palette_variant = parent.palette_variant.clone();
        }

        // Prepend parent effects (child effects applied after parent)
        let mut combined = parent.effects.clone();
        combined.append(&mut self.effects);
        self.effects = combined;
    }
}

/// A post-processing effect.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Darkens the edges of the image.
    Vignette {
        /// Strength of the effect (0.0 - 1.0).
        strength: f32,
    },

    /// Adds horizontal scanlines.
    Scanlines {
        /// Opacity of the scanlines (0.0 - 1.0).
        opacity: f32,
        /// Gap between scanlines in pixels (default: 2).
        gap: u32,
    },

    /// Adjusts overall brightness.
    Brightness {
        /// Brightness adjustment (-1.0 to 1.0, 0 = no change).
        amount: f32,
    },

    /// Adjusts contrast.
    Contrast {
        /// Contrast adjustment (-1.0 to 1.0, 0 = no change).
        amount: f32,
    },

    /// Custom/unknown effect with raw parameters.
    Custom {
        /// Effect type name.
        name: String,
        /// Raw parameters.
        params: HashMap<String, EffectParam>,
    },
}

/// A parameter value for effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectParam {
    /// Floating point number.
    Float(f32),
    /// Integer.
    Int(i64),
    /// String value.
    String(String),
    /// Boolean.
    Bool(bool),
}

impl Effect {
    /// Create a vignette effect.
    pub fn vignette(strength: f32) -> Self {
        Self::Vignette {
            strength: strength.clamp(0.0, 1.0),
        }
    }

    /// Create a scanlines effect.
    pub fn scanlines(opacity: f32) -> Self {
        Self::Scanlines {
            opacity: opacity.clamp(0.0, 1.0),
            gap: 2,
        }
    }

    /// Create a scanlines effect with custom gap.
    pub fn scanlines_with_gap(opacity: f32, gap: u32) -> Self {
        Self::Scanlines {
            opacity: opacity.clamp(0.0, 1.0),
            gap,
        }
    }

    /// Create a brightness effect.
    pub fn brightness(amount: f32) -> Self {
        Self::Brightness {
            amount: amount.clamp(-1.0, 1.0),
        }
    }

    /// Create a contrast effect.
    pub fn contrast(amount: f32) -> Self {
        Self::Contrast {
            amount: amount.clamp(-1.0, 1.0),
        }
    }

    /// Get the effect type name.
    pub fn type_name(&self) -> &str {
        match self {
            Effect::Vignette { .. } => "vignette",
            Effect::Scanlines { .. } => "scanlines",
            Effect::Brightness { .. } => "brightness",
            Effect::Contrast { .. } => "contrast",
            Effect::Custom { name, .. } => name,
        }
    }
}

/// Collection of builtin shaders.
pub struct BuiltinShaders;

impl BuiltinShaders {
    /// Get the default shader (binds to default palette, no effects).
    pub fn default_shader() -> Shader {
        Shader::new("default", "default")
    }

    /// Get all builtin shaders.
    pub fn all() -> Vec<Shader> {
        vec![Self::default_shader()]
    }

    /// Get a builtin shader by name.
    pub fn get(name: &str) -> Option<Shader> {
        Self::all().into_iter().find(|s| s.name == name)
    }
}

/// Builder for constructing shaders from parsed definitions.
#[derive(Debug, Clone)]
pub struct ShaderBuilder {
    name: String,
    palette: Option<String>,
    palette_variant: Option<String>,
    effects: Vec<Effect>,
    inherits: Option<String>,
}

impl ShaderBuilder {
    /// Create a new shader builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            palette: None,
            palette_variant: None,
            effects: Vec::new(),
            inherits: None,
        }
    }

    /// Set the palette.
    pub fn palette(&mut self, palette: impl Into<String>) -> &mut Self {
        self.palette = Some(palette.into());
        self
    }

    /// Set the palette variant.
    pub fn palette_variant(&mut self, variant: impl Into<String>) -> &mut Self {
        self.palette_variant = Some(variant.into());
        self
    }

    /// Add an effect.
    pub fn add_effect(&mut self, effect: Effect) -> &mut Self {
        self.effects.push(effect);
        self
    }

    /// Set the parent shader for inheritance.
    pub fn inherits(&mut self, parent: impl Into<String>) -> &mut Self {
        self.inherits = Some(parent.into());
        self
    }

    /// Get the parent shader name if set.
    pub fn parent_name(&self) -> Option<&str> {
        self.inherits.as_deref()
    }

    /// Build the shader.
    ///
    /// If `parent` is provided, settings will be inherited from it.
    /// Returns an error if no palette is specified and there's no parent.
    pub fn build(self, parent: Option<&Shader>) -> Result<Shader, &'static str> {
        // Determine palette (required)
        let palette = self
            .palette
            .or_else(|| parent.map(|p| p.palette.clone()))
            .ok_or("Shader must specify a palette")?;

        let mut shader = Shader {
            name: self.name,
            palette,
            palette_variant: self.palette_variant,
            effects: self.effects,
            inherits: self.inherits,
        };

        // Merge from parent if provided
        if let Some(parent) = parent {
            shader.merge_from(parent);
        }

        Ok(shader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_new() {
        let shader = Shader::new("test", "my-palette");
        assert_eq!(shader.name, "test");
        assert_eq!(shader.palette, "my-palette");
        assert_eq!(shader.palette_variant, None);
        assert!(shader.effects.is_empty());
    }

    #[test]
    fn test_shader_with_variant() {
        let shader = Shader::new("test", "dungeon").with_variant("dark-mode");
        assert_eq!(shader.palette_variant, Some("dark-mode".to_string()));
    }

    #[test]
    fn test_shader_with_effects() {
        let shader = Shader::new("test", "palette")
            .with_effect(Effect::vignette(0.3))
            .with_effect(Effect::scanlines(0.1));

        assert_eq!(shader.effects.len(), 2);
        assert!(shader.has_effects());
    }

    #[test]
    fn test_effect_vignette() {
        let effect = Effect::vignette(0.5);
        assert_eq!(effect.type_name(), "vignette");
        if let Effect::Vignette { strength } = effect {
            assert_eq!(strength, 0.5);
        } else {
            panic!("Expected Vignette");
        }
    }

    #[test]
    fn test_effect_scanlines() {
        let effect = Effect::scanlines(0.2);
        assert_eq!(effect.type_name(), "scanlines");
        if let Effect::Scanlines { opacity, gap } = effect {
            assert_eq!(opacity, 0.2);
            assert_eq!(gap, 2);
        } else {
            panic!("Expected Scanlines");
        }
    }

    #[test]
    fn test_effect_clamping() {
        let effect = Effect::vignette(2.0); // Should clamp to 1.0
        if let Effect::Vignette { strength } = effect {
            assert_eq!(strength, 1.0);
        }

        let effect = Effect::brightness(-2.0); // Should clamp to -1.0
        if let Effect::Brightness { amount } = effect {
            assert_eq!(amount, -1.0);
        }
    }

    #[test]
    fn test_builtin_default_shader() {
        let shader = BuiltinShaders::default_shader();
        assert_eq!(shader.name, "default");
        assert_eq!(shader.palette, "default");
        assert!(!shader.has_effects());
    }

    #[test]
    fn test_shader_builder() {
        let mut builder = ShaderBuilder::new("test");
        builder.palette("my-palette");
        builder.palette_variant("dark");
        builder.add_effect(Effect::vignette(0.5));

        let shader = builder.build(None).unwrap();
        assert_eq!(shader.name, "test");
        assert_eq!(shader.palette, "my-palette");
        assert_eq!(shader.palette_variant, Some("dark".to_string()));
        assert_eq!(shader.effects.len(), 1);
    }

    #[test]
    fn test_shader_builder_no_palette() {
        let builder = ShaderBuilder::new("test");
        let result = builder.build(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_shader_builder_inherits_palette() {
        let parent = Shader::new("parent", "parent-palette");

        let mut builder = ShaderBuilder::new("child");
        builder.inherits("parent");
        // No palette specified - should inherit from parent

        let shader = builder.build(Some(&parent)).unwrap();
        assert_eq!(shader.palette, "parent-palette");
    }

    #[test]
    fn test_shader_merge_effects() {
        let parent = Shader::new("parent", "palette").with_effect(Effect::vignette(0.2));

        let mut child = Shader::new("child", "palette").with_effect(Effect::scanlines(0.1));

        child.merge_from(&parent);

        // Parent effects come first, then child effects
        assert_eq!(child.effects.len(), 2);
        assert_eq!(child.effects[0].type_name(), "vignette");
        assert_eq!(child.effects[1].type_name(), "scanlines");
    }

    #[test]
    fn test_shader_inherit_variant() {
        let parent = Shader::new("parent", "palette").with_variant("dark");

        let mut child = Shader::new("child", "palette");
        // No variant set on child

        child.merge_from(&parent);

        assert_eq!(child.palette_variant, Some("dark".to_string()));
    }

    #[test]
    fn test_shader_child_variant_overrides() {
        let parent = Shader::new("parent", "palette").with_variant("dark");

        let mut child = Shader::new("child", "palette").with_variant("light");

        child.merge_from(&parent);

        // Child's variant should take precedence
        assert_eq!(child.palette_variant, Some("light".to_string()));
    }
}
