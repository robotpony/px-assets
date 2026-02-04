//! Shader file parser.
//!
//! Parses `.shader.md` files into `Shader` instances.

use std::collections::HashMap;

use crate::error::Result;
use crate::parser::{parse_documents, RawDocument};
use crate::types::{Effect, EffectParam, ShaderBuilder};

/// Parse a shader file into one or more shaders.
///
/// Each document in the file becomes a separate shader.
pub fn parse_shader_file(source: &str) -> Result<Vec<ShaderBuilder>> {
    let documents = parse_documents(source)?;

    documents
        .into_iter()
        .map(parse_shader_document)
        .collect()
}

/// Parse a single shader document.
fn parse_shader_document(doc: RawDocument) -> Result<ShaderBuilder> {
    let name = doc.name.value.clone();
    let mut builder = ShaderBuilder::new(name.clone());

    // Get palette (required in frontmatter)
    if let Some(palette) = doc.get_frontmatter_str("palette") {
        builder.palette(palette);
    }

    // Get palette variant (optional)
    if let Some(variant) = doc.get_frontmatter_str("palette_variant") {
        builder.palette_variant(variant);
    }

    // Get inheritance (optional)
    if let Some(inherits) = doc.get_frontmatter_str("inherits") {
        builder.inherits(inherits);
    }

    // Parse effects from frontmatter (if present as YAML)
    if let Some(effects_value) = doc.get_frontmatter("effects") {
        if let Some(effects) = effects_value.value.as_sequence() {
            for effect_value in effects {
                if let Some(effect) = parse_effect(effect_value) {
                    builder.add_effect(effect);
                }
            }
        }
    }

    // Also try parsing effects from body (key-value format)
    if let Some(body) = &doc.body {
        parse_body_effects(&body.value, &mut builder)?;
    }

    Ok(builder)
}

/// Parse an effect from a YAML value.
fn parse_effect(value: &serde_yaml::Value) -> Option<Effect> {
    let map = value.as_mapping()?;

    // Get effect type
    let effect_type = map
        .get(&serde_yaml::Value::String("type".to_string()))?
        .as_str()?;

    match effect_type {
        "vignette" => {
            let strength = map
                .get(&serde_yaml::Value::String("strength".to_string()))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.3) as f32;
            Some(Effect::vignette(strength))
        }
        "scanlines" => {
            let opacity = map
                .get(&serde_yaml::Value::String("opacity".to_string()))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.1) as f32;
            let gap = map
                .get(&serde_yaml::Value::String("gap".to_string()))
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as u32;
            Some(Effect::scanlines_with_gap(opacity, gap))
        }
        "brightness" => {
            let amount = map
                .get(&serde_yaml::Value::String("amount".to_string()))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            Some(Effect::brightness(amount))
        }
        "contrast" => {
            let amount = map
                .get(&serde_yaml::Value::String("amount".to_string()))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            Some(Effect::contrast(amount))
        }
        _ => {
            // Unknown effect - store as custom
            let mut params = HashMap::new();
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str != "type" {
                        if let Some(param) = yaml_to_param(val) {
                            params.insert(key_str.to_string(), param);
                        }
                    }
                }
            }
            Some(Effect::Custom {
                name: effect_type.to_string(),
                params,
            })
        }
    }
}

/// Convert a YAML value to an effect parameter.
fn yaml_to_param(value: &serde_yaml::Value) -> Option<EffectParam> {
    match value {
        serde_yaml::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Some(EffectParam::Float(f as f32))
            } else if let Some(i) = n.as_i64() {
                Some(EffectParam::Int(i))
            } else {
                None
            }
        }
        serde_yaml::Value::String(s) => Some(EffectParam::String(s.clone())),
        serde_yaml::Value::Bool(b) => Some(EffectParam::Bool(*b)),
        _ => None,
    }
}

/// Parse effects from the body content (key-value format).
///
/// The body can contain lines like:
/// - `lighting: ambient`
/// - `ambient_color: $dark`
///
/// For now, these are informational and not converted to effects.
fn parse_body_effects(body: &str, _builder: &mut ShaderBuilder) -> Result<()> {
    // Body content in shaders is typically comments or additional
    // configuration that we don't need to parse for basic functionality.
    // Effects are primarily defined in frontmatter YAML.
    //
    // Future: could parse key-value pairs here for extended config.

    for line in body.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Could parse key: value pairs here in the future
        // For now, we just acknowledge the body exists
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_shader() {
        let source = r#"---
name: test
palette: my-palette
---
"#;

        let builders = parse_shader_file(source).unwrap();
        assert_eq!(builders.len(), 1);

        let shader = builders[0].clone().build(None).unwrap();
        assert_eq!(shader.name, "test");
        assert_eq!(shader.palette, "my-palette");
        assert!(shader.effects.is_empty());
    }

    #[test]
    fn test_parse_shader_with_variant() {
        let source = r#"---
name: test
palette: dungeon
palette_variant: dark-mode
---
"#;

        let builders = parse_shader_file(source).unwrap();
        let shader = builders[0].clone().build(None).unwrap();

        assert_eq!(shader.palette, "dungeon");
        assert_eq!(shader.palette_variant, Some("dark-mode".to_string()));
    }

    #[test]
    fn test_parse_shader_with_effects() {
        let source = r#"---
name: test
palette: dungeon
effects:
  - type: vignette
    strength: 0.3
  - type: scanlines
    opacity: 0.1
---
"#;

        let builders = parse_shader_file(source).unwrap();
        let shader = builders[0].clone().build(None).unwrap();

        assert_eq!(shader.effects.len(), 2);
        assert_eq!(shader.effects[0].type_name(), "vignette");
        assert_eq!(shader.effects[1].type_name(), "scanlines");
    }

    #[test]
    fn test_parse_shader_with_inheritance() {
        let source = r#"---
name: child
palette: my-palette
inherits: parent
---
"#;

        let builders = parse_shader_file(source).unwrap();
        assert_eq!(builders[0].parent_name(), Some("parent"));
    }

    #[test]
    fn test_parse_multiple_shaders() {
        let source = r#"---
name: shader-a
palette: palette-a
---

---
name: shader-b
palette: palette-b
---
"#;

        let builders = parse_shader_file(source).unwrap();
        assert_eq!(builders.len(), 2);
        assert_eq!(builders[0].clone().build(None).unwrap().name, "shader-a");
        assert_eq!(builders[1].clone().build(None).unwrap().name, "shader-b");
    }

    #[test]
    fn test_parse_default_shader() {
        let source = r#"---
name: default
palette: default
---

# No effects, just palette binding
"#;

        let builders = parse_shader_file(source).unwrap();
        let shader = builders[0].clone().build(None).unwrap();

        assert_eq!(shader.name, "default");
        assert_eq!(shader.palette, "default");
        assert!(shader.effects.is_empty());
    }

    #[test]
    fn test_parse_custom_effect() {
        let source = r#"---
name: test
palette: test
effects:
  - type: custom-blur
    radius: 5
    enabled: true
---
"#;

        let builders = parse_shader_file(source).unwrap();
        let shader = builders[0].clone().build(None).unwrap();

        assert_eq!(shader.effects.len(), 1);
        if let Effect::Custom { name, params } = &shader.effects[0] {
            assert_eq!(name, "custom-blur");
            assert!(params.contains_key("radius"));
            assert!(params.contains_key("enabled"));
        } else {
            panic!("Expected Custom effect");
        }
    }

    #[test]
    fn test_parse_vignette_default_strength() {
        let source = r#"---
name: test
palette: test
effects:
  - type: vignette
---
"#;

        let builders = parse_shader_file(source).unwrap();
        let shader = builders[0].clone().build(None).unwrap();

        if let Effect::Vignette { strength } = shader.effects[0] {
            assert_eq!(strength, 0.3); // Default
        } else {
            panic!("Expected Vignette");
        }
    }

    #[test]
    fn test_parse_scanlines_with_gap() {
        let source = r#"---
name: test
palette: test
effects:
  - type: scanlines
    opacity: 0.2
    gap: 4
---
"#;

        let builders = parse_shader_file(source).unwrap();
        let shader = builders[0].clone().build(None).unwrap();

        if let Effect::Scanlines { opacity, gap } = shader.effects[0] {
            assert_eq!(opacity, 0.2);
            assert_eq!(gap, 4);
        } else {
            panic!("Expected Scanlines");
        }
    }
}
