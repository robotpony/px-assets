//! Palette type for named colour collections.

use std::collections::{HashMap, HashSet};

use crate::error::{PxError, Result};

use super::expr::{ColourExpr, ExprEvaluator};
use super::Colour;

/// A collection of named colours with optional variants.
#[derive(Debug, Clone)]
pub struct Palette {
    /// Palette name
    pub name: String,

    /// Base colours (resolved to actual RGBA values)
    colours: HashMap<String, Colour>,

    /// Variant overrides (variant_name -> colour_name -> colour)
    variants: HashMap<String, HashMap<String, Colour>>,
}

impl Palette {
    /// Create a new empty palette.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            colours: HashMap::new(),
            variants: HashMap::new(),
        }
    }

    /// Create the builtin default palette.
    pub fn default_palette() -> Self {
        let mut palette = Self::new("default");
        palette.colours.insert("black".to_string(), Colour::BLACK);
        palette.colours.insert("white".to_string(), Colour::WHITE);
        palette.colours.insert("edge".to_string(), Colour::BLACK);
        palette.colours.insert("fill".to_string(), Colour::WHITE);
        palette
    }

    /// Get a colour by name.
    pub fn get(&self, name: &str) -> Option<Colour> {
        // Strip $ prefix if present
        let name = name.strip_prefix('$').unwrap_or(name);
        self.colours.get(name).copied()
    }

    /// Get a colour with variant applied.
    pub fn get_with_variant(&self, name: &str, variant: &str) -> Option<Colour> {
        let name = name.strip_prefix('$').unwrap_or(name);

        // Check variant first, fall back to base
        self.variants
            .get(variant)
            .and_then(|v| v.get(name))
            .copied()
            .or_else(|| self.colours.get(name).copied())
    }

    /// Get all colour names.
    pub fn colour_names(&self) -> impl Iterator<Item = &str> {
        self.colours.keys().map(|s| s.as_str())
    }

    /// Get all variant names.
    pub fn variant_names(&self) -> impl Iterator<Item = &str> {
        self.variants.keys().map(|s| s.as_str())
    }

    /// Check if the palette has a variant.
    pub fn has_variant(&self, name: &str) -> bool {
        self.variants.contains_key(name)
    }

    /// Get the number of colours.
    pub fn len(&self) -> usize {
        self.colours.len()
    }

    /// Check if the palette is empty.
    pub fn is_empty(&self) -> bool {
        self.colours.is_empty()
    }

    /// Insert a resolved colour.
    pub(crate) fn insert(&mut self, name: String, colour: Colour) {
        self.colours.insert(name, colour);
    }

    /// Insert a variant colour override.
    pub(crate) fn insert_variant(&mut self, variant: String, name: String, colour: Colour) {
        self.variants
            .entry(variant)
            .or_default()
            .insert(name, colour);
    }

    /// Merge another palette into this one (for inheritance).
    pub fn merge_from(&mut self, other: &Palette) {
        // Copy base colours (don't overwrite existing)
        for (name, colour) in &other.colours {
            self.colours.entry(name.clone()).or_insert(*colour);
        }

        // Copy variants (don't overwrite existing)
        for (variant_name, colours) in &other.variants {
            let entry = self.variants.entry(variant_name.clone()).or_default();
            for (name, colour) in colours {
                entry.entry(name.clone()).or_insert(*colour);
            }
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::default_palette()
    }
}

/// Builder for constructing palettes from parsed definitions.
#[derive(Debug)]
pub struct PaletteBuilder {
    name: String,
    /// Unresolved definitions: name -> value (hex or reference)
    definitions: Vec<(String, ColourDef)>,
    /// Variant definitions: variant_name -> [(colour_name, value)]
    variant_defs: HashMap<String, Vec<(String, ColourDef)>>,
    /// Parent palette name for inheritance
    inherits: Option<String>,
}

#[derive(Debug, Clone)]
enum ColourDef {
    /// A simple hex value: #FF0000
    Hex(String),
    /// A reference to another colour: $dark
    Reference(String),
    /// A colour expression: darken($gold, 20%)
    Expression(ColourExpr),
}

impl PaletteBuilder {
    /// Create a new palette builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            definitions: Vec::new(),
            variant_defs: HashMap::new(),
            inherits: None,
        }
    }

    /// Add a colour definition (hex, reference, or expression).
    pub fn define(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();

        let def = parse_colour_def(&value);
        self.definitions.push((name, def));
    }

    /// Add a variant colour override.
    pub fn define_variant(
        &mut self,
        variant: impl Into<String>,
        name: impl Into<String>,
        value: impl Into<String>,
    ) {
        let variant = variant.into();
        let name = name.into();
        let value = value.into();

        let def = parse_colour_def(&value);

        self.variant_defs
            .entry(variant)
            .or_default()
            .push((name, def));
    }

    /// Set the parent palette for inheritance.
    pub fn inherits(&mut self, parent: impl Into<String>) {
        self.inherits = Some(parent.into());
    }

    /// Get the parent palette name if set.
    pub fn parent_name(&self) -> Option<&str> {
        self.inherits.as_deref()
    }

    /// Build the palette, resolving all references.
    ///
    /// If `parent` is provided, colours will be inherited from it.
    pub fn build(self, parent: Option<&Palette>) -> Result<Palette> {
        let mut palette = Palette::new(self.name.clone());

        // Inherit from parent first
        if let Some(parent) = parent {
            palette.merge_from(parent);
        }

        // Resolve base colours
        let resolved = resolve_colours(&self.definitions, &palette)?;
        for (name, colour) in resolved {
            palette.insert(name, colour);
        }

        // Resolve variant colours
        for (variant_name, defs) in self.variant_defs {
            let resolved = resolve_colours(&defs, &palette)?;
            for (name, colour) in resolved {
                palette.insert_variant(variant_name.clone(), name, colour);
            }
        }

        Ok(palette)
    }
}

/// Parse a colour value string into a ColourDef.
fn parse_colour_def(value: &str) -> ColourDef {
    let value = value.trim();

    // Check if it's a function call (contains parentheses)
    if value.contains('(') {
        // Parse as expression
        match ColourExpr::parse(value) {
            Ok(expr) => ColourDef::Expression(expr),
            // Fall back to treating as hex if parse fails
            Err(_) => ColourDef::Hex(value.to_string()),
        }
    } else if value.starts_with('$') {
        ColourDef::Reference(value.to_string())
    } else {
        ColourDef::Hex(value.to_string())
    }
}

/// Resolve a list of colour definitions, handling references and expressions.
fn resolve_colours(
    definitions: &[(String, ColourDef)],
    existing: &Palette,
) -> Result<Vec<(String, Colour)>> {
    // Build a map of definitions for lookup
    let def_map: HashMap<&str, &ColourDef> = definitions
        .iter()
        .map(|(name, def)| (name.as_str(), def))
        .collect();

    let mut resolved: HashMap<String, Colour> = HashMap::new();
    let mut resolving: HashSet<String> = HashSet::new();

    for (name, _) in definitions {
        resolve_single(name, &def_map, existing, &mut resolved, &mut resolving)?;
    }

    Ok(definitions
        .iter()
        .filter_map(|(name, _)| resolved.remove(name).map(|c| (name.clone(), c)))
        .collect())
}

/// Resolve a single colour definition, detecting cycles.
fn resolve_single(
    name: &str,
    definitions: &HashMap<&str, &ColourDef>,
    existing: &Palette,
    resolved: &mut HashMap<String, Colour>,
    resolving: &mut HashSet<String>,
) -> Result<Colour> {
    // Already resolved?
    if let Some(&colour) = resolved.get(name) {
        return Ok(colour);
    }

    // Check for cycles
    if resolving.contains(name) {
        return Err(PxError::Parse {
            message: format!("Circular colour reference: ${}", name),
            help: Some("Check your colour definitions for circular references".to_string()),
        });
    }

    // Get definition
    let def = match definitions.get(name) {
        Some(def) => def,
        None => {
            // Try existing palette (from inheritance)
            return existing.get(name).ok_or_else(|| PxError::Parse {
                message: format!("Undefined colour: ${}", name),
                help: None,
            });
        }
    };

    resolving.insert(name.to_string());

    let colour = match def {
        ColourDef::Hex(hex) => Colour::from_hex(hex)?,
        ColourDef::Reference(ref_name) => {
            let ref_name = ref_name.strip_prefix('$').unwrap_or(ref_name);
            resolve_single(ref_name, definitions, existing, resolved, resolving)?
        }
        ColourDef::Expression(expr) => {
            // Create an evaluator that can look up colours
            let evaluator = ExprEvaluator::new(|ref_name| {
                // Try resolved colours first
                if let Some(&c) = resolved.get(ref_name) {
                    return Some(c);
                }
                // Then try to resolve the reference
                if let Some(def) = definitions.get(ref_name) {
                    // For simple cases, resolve inline
                    match def {
                        ColourDef::Hex(hex) => Colour::from_hex(hex).ok(),
                        ColourDef::Reference(r) => {
                            let r = r.strip_prefix('$').unwrap_or(r);
                            existing.get(r)
                        }
                        ColourDef::Expression(_) => {
                            // Complex case: the reference itself is an expression
                            // This requires recursive resolution
                            None
                        }
                    }
                } else {
                    // Try existing palette
                    existing.get(ref_name)
                }
            });
            evaluator.eval(expr)?
        }
    };

    resolving.remove(name);
    resolved.insert(name.to_string(), colour);

    Ok(colour)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_new() {
        let palette = Palette::new("test");
        assert_eq!(palette.name, "test");
        assert!(palette.is_empty());
    }

    #[test]
    fn test_palette_default() {
        let palette = Palette::default_palette();
        assert_eq!(palette.get("black"), Some(Colour::BLACK));
        assert_eq!(palette.get("white"), Some(Colour::WHITE));
        assert_eq!(palette.get("edge"), Some(Colour::BLACK));
        assert_eq!(palette.get("fill"), Some(Colour::WHITE));
    }

    #[test]
    fn test_palette_get_with_dollar() {
        let palette = Palette::default_palette();
        assert_eq!(palette.get("$black"), Some(Colour::BLACK));
    }

    #[test]
    fn test_builder_hex_colours() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("red", "#FF0000");
        builder.define("green", "#00FF00");

        let palette = builder.build(None).unwrap();

        assert_eq!(palette.get("red"), Some(Colour::rgb(255, 0, 0)));
        assert_eq!(palette.get("green"), Some(Colour::rgb(0, 255, 0)));
    }

    #[test]
    fn test_builder_references() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("dark", "#1a1a1a");
        builder.define("edge", "$dark");

        let palette = builder.build(None).unwrap();

        assert_eq!(palette.get("dark"), palette.get("edge"));
    }

    #[test]
    fn test_builder_chained_references() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("base", "#FF0000");
        builder.define("primary", "$base");
        builder.define("accent", "$primary");

        let palette = builder.build(None).unwrap();

        assert_eq!(palette.get("accent"), Some(Colour::rgb(255, 0, 0)));
    }

    #[test]
    fn test_builder_circular_reference() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("a", "$b");
        builder.define("b", "$a");

        let result = builder.build(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_variants() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("dark", "#000000");
        builder.define_variant("light-mode", "dark", "#FFFFFF");

        let palette = builder.build(None).unwrap();

        assert_eq!(palette.get("dark"), Some(Colour::BLACK));
        assert_eq!(
            palette.get_with_variant("dark", "light-mode"),
            Some(Colour::WHITE)
        );
    }

    #[test]
    fn test_builder_inheritance() {
        let parent = Palette::default_palette();

        let mut builder = PaletteBuilder::new("child");
        builder.define("custom", "#FF0000");
        builder.define("edge", "$custom"); // Override parent's edge

        let palette = builder.build(Some(&parent)).unwrap();

        // Inherited from parent
        assert_eq!(palette.get("black"), Some(Colour::BLACK));
        // New definition
        assert_eq!(palette.get("custom"), Some(Colour::rgb(255, 0, 0)));
        // Overridden
        assert_eq!(palette.get("edge"), Some(Colour::rgb(255, 0, 0)));
    }

    #[test]
    fn test_variant_fallback() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("dark", "#000000");
        builder.define("light", "#FFFFFF");
        builder.define_variant("alt", "dark", "#333333");
        // Note: no override for "light" in "alt" variant

        let palette = builder.build(None).unwrap();

        // Variant override
        assert_eq!(
            palette.get_with_variant("dark", "alt"),
            Some(Colour::rgb(0x33, 0x33, 0x33))
        );
        // Fallback to base
        assert_eq!(
            palette.get_with_variant("light", "alt"),
            Some(Colour::WHITE)
        );
    }

    #[test]
    fn test_builder_darken_expression() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("base", "#FFFFFF");
        builder.define("dark", "darken($base, 50%)");

        let palette = builder.build(None).unwrap();

        // White darkened by 50% should be grey
        let dark = palette.get("dark").unwrap();
        assert!(dark.r < 255);
        assert!(dark.g < 255);
        assert!(dark.b < 255);
        // Should be greyscale
        assert_eq!(dark.r, dark.g);
        assert_eq!(dark.g, dark.b);
    }

    #[test]
    fn test_builder_lighten_expression() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("base", "#000000");
        builder.define("light", "lighten($base, 50%)");

        let palette = builder.build(None).unwrap();

        // Black lightened by 50% should be grey
        let light = palette.get("light").unwrap();
        assert!(light.r > 0);
        assert!(light.g > 0);
        assert!(light.b > 0);
    }

    #[test]
    fn test_builder_mix_expression() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("black", "#000000");
        builder.define("white", "#FFFFFF");
        builder.define("grey", "mix($black, $white, 50%)");

        let palette = builder.build(None).unwrap();

        let grey = palette.get("grey").unwrap();
        assert_eq!(grey.r, 128);
        assert_eq!(grey.g, 128);
        assert_eq!(grey.b, 128);
    }

    #[test]
    fn test_builder_alpha_expression() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("red", "#FF0000");
        builder.define("transparent_red", "alpha($red, 50%)");

        let palette = builder.build(None).unwrap();

        let tr = palette.get("transparent_red").unwrap();
        assert_eq!(tr.r, 255);
        assert_eq!(tr.g, 0);
        assert_eq!(tr.b, 0);
        // 50% of 255 = 127.5, rounds to 127 or 128
        assert!(tr.a == 127 || tr.a == 128);
    }

    #[test]
    fn test_builder_saturate_expression() {
        let mut builder = PaletteBuilder::new("test");
        // Use a colour with existing saturation
        builder.define("muted", "#B08080"); // Desaturated red
        builder.define("vivid", "saturate($muted, 50%)");

        let palette = builder.build(None).unwrap();

        // Saturating should increase the difference between r and g/b
        let muted = palette.get("muted").unwrap();
        let vivid = palette.get("vivid").unwrap();

        // The red channel should stay dominant
        assert!(vivid.r > vivid.g);
        assert!(vivid.r > vivid.b);
        // And should be more saturated (bigger gap)
        let muted_gap = muted.r as i32 - muted.g as i32;
        let vivid_gap = vivid.r as i32 - vivid.g as i32;
        assert!(vivid_gap >= muted_gap);
    }

    #[test]
    fn test_builder_desaturate_expression() {
        let mut builder = PaletteBuilder::new("test");
        builder.define("red", "#FF0000");
        builder.define("muted", "desaturate($red, 100%)");

        let palette = builder.build(None).unwrap();

        // Fully desaturated should be greyscale
        let muted = palette.get("muted").unwrap();
        assert_eq!(muted.r, muted.g);
        assert_eq!(muted.g, muted.b);
    }

    #[test]
    fn test_builder_expression_with_inherited_ref() {
        let parent = Palette::default_palette();

        let mut builder = PaletteBuilder::new("child");
        // Use inherited $black in expression
        builder.define("dark_grey", "lighten($black, 20%)");

        let palette = builder.build(Some(&parent)).unwrap();

        let dark_grey = palette.get("dark_grey").unwrap();
        assert!(dark_grey.r > 0);
        assert!(dark_grey.r < 128);
    }
}
