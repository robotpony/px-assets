//! Target file parser.
//!
//! Parses `.target.md` files into `Target` instances.
//! Like palettes, targets use plain key-value lines outside of code fences,
//! so we bypass `parse_documents` and handle frontmatter extraction directly.

use crate::error::{PxError, Result};
use crate::types::{PaletteMode, SheetConfig, TargetBuilder};

/// Parse a target file into one or more targets.
///
/// Each document in the file becomes a separate target.
pub fn parse_target_file(source: &str) -> Result<Vec<TargetBuilder>> {
    let sections = split_target_sections(source);

    let mut builders = Vec::new();

    for section in sections {
        let builder = parse_single_target(&section)?;
        builders.push(builder);
    }

    if builders.is_empty() {
        return Err(PxError::Parse {
            message: "No target definitions found".to_string(),
            help: Some("Add a target with ---\\nname: my-target\\n---".to_string()),
        });
    }

    Ok(builders)
}

/// Split source into target sections (for multi-target files).
fn split_target_sections(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut sections = Vec::new();
    let mut current_start = 0;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line == "---" && i > 0 {
            if let Some(next_line) = lines.get(i + 1) {
                if next_line.trim().starts_with("name:") {
                    let section: String = lines[current_start..i]
                        .iter()
                        .map(|l| format!("{}\n", l))
                        .collect();
                    if !section.trim().is_empty() {
                        sections.push(section);
                    }
                    current_start = i;
                }
            }
        }
        i += 1;
    }

    // Add final section
    let section: String = lines[current_start..]
        .iter()
        .map(|l| format!("{}\n", l))
        .collect();
    if !section.trim().is_empty() {
        sections.push(section);
    }

    sections
}

/// Parse a single target section.
fn parse_single_target(source: &str) -> Result<TargetBuilder> {
    let trimmed = source.trim();

    if !trimmed.starts_with("---") {
        return Err(PxError::Parse {
            message: "Target must start with ---".to_string(),
            help: Some("Add YAML frontmatter: ---\\nname: my-target\\n---".to_string()),
        });
    }

    // Find closing ---
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(after_opening);
    let after_opening = after_opening.strip_prefix('\r').unwrap_or(after_opening);

    let mut closing_pos = None;
    let mut pos = 0;
    for line in after_opening.lines() {
        if line.trim() == "---" {
            closing_pos = Some(pos);
            break;
        }
        pos += line.len() + 1;
    }

    let closing_pos = closing_pos.ok_or_else(|| PxError::Parse {
        message: "Unclosed frontmatter".to_string(),
        help: Some("Add --- after the YAML content".to_string()),
    })?;

    let frontmatter_content = &after_opening[..closing_pos].trim();
    let body_start = closing_pos + 3; // Skip past "---"
    let body = if body_start < after_opening.len() {
        &after_opening[body_start..]
    } else {
        ""
    };

    // Parse frontmatter as YAML
    let frontmatter: serde_yaml::Value =
        serde_yaml::from_str(frontmatter_content).map_err(|e| PxError::Parse {
            message: format!("Invalid YAML: {}", e),
            help: None,
        })?;

    // Get name (required)
    let name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PxError::Parse {
            message: "Target missing required 'name' field".to_string(),
            help: None,
        })?;

    let mut builder = TargetBuilder::new(name);

    // Read fields from frontmatter
    if let Some(format) = frontmatter.get("format").and_then(|v| v.as_str()) {
        builder.format(format);
    }
    if let Some(scale) = frontmatter.get("scale").and_then(|v| v.as_u64()) {
        builder.scale(scale as u32);
    }
    if let Some(sheet) = frontmatter.get("sheet").and_then(|v| v.as_str()) {
        if let Ok(config) = SheetConfig::parse(sheet) {
            builder.sheet(config);
        }
    }
    if let Some(padding) = frontmatter.get("padding").and_then(|v| v.as_u64()) {
        builder.padding(padding as u32);
    }
    if let Some(mode) = frontmatter.get("palette_mode").and_then(|v| v.as_str()) {
        match mode {
            "indexed" => { builder.palette_mode(PaletteMode::Indexed); }
            _ => { builder.palette_mode(PaletteMode::Rgba); }
        }
    }
    if let Some(shader) = frontmatter.get("shader").and_then(|v| v.as_str()) {
        builder.shader(shader);
    }

    // Parse body key-value lines (body values override frontmatter)
    parse_target_body(body, &mut builder)?;

    Ok(builder)
}

/// Parse key-value pairs from the body content.
///
/// Supports lines like:
/// - `scale: 2`
/// - `sheet: auto`
/// - `padding: 1`
/// - `shader: dark`
/// - `palette_mode: indexed`
///
/// Unknown keys are ignored for forward compatibility.
fn parse_target_body(body: &str, builder: &mut TargetBuilder) -> Result<()> {
    for line in body.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Parse key: value
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "format" => {
                    builder.format(value);
                }
                "scale" => {
                    if let Ok(n) = value.parse::<u32>() {
                        builder.scale(n);
                    }
                }
                "sheet" => {
                    if let Ok(config) = SheetConfig::parse(value) {
                        builder.sheet(config);
                    }
                }
                "padding" => {
                    if let Ok(n) = value.parse::<u32>() {
                        builder.padding(n);
                    }
                }
                "palette_mode" => match value {
                    "indexed" => {
                        builder.palette_mode(PaletteMode::Indexed);
                    }
                    _ => {
                        builder.palette_mode(PaletteMode::Rgba);
                    }
                },
                "shader" => {
                    builder.shader(value);
                }
                // Unknown keys ignored for forward compat (tile, colors, etc.)
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SheetConfig;

    #[test]
    fn test_parse_simple_target() {
        let source = r#"---
name: web
format: png
---
"#;

        let builders = parse_target_file(source).unwrap();
        assert_eq!(builders.len(), 1);

        let target = builders[0].clone().build().unwrap();
        assert_eq!(target.name, "web");
        assert_eq!(target.format, "png");
    }

    #[test]
    fn test_parse_target_with_body() {
        let source = r#"---
name: retro
format: png
---

scale: 4
sheet: auto
padding: 2
shader: dark
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        assert_eq!(target.name, "retro");
        assert_eq!(target.scale, Some(4));
        assert_eq!(target.sheet, SheetConfig::Auto);
        assert_eq!(target.padding, Some(2));
        assert_eq!(target.shader, Some("dark".to_string()));
    }

    #[test]
    fn test_parse_target_frontmatter_fields() {
        let source = r#"---
name: compact
format: png
scale: 2
sheet: 8x4
padding: 1
---
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        assert_eq!(target.scale, Some(2));
        assert_eq!(
            target.sheet,
            SheetConfig::Fixed {
                width: 8,
                height: 4
            }
        );
        assert_eq!(target.padding, Some(1));
    }

    #[test]
    fn test_parse_target_body_overrides_frontmatter() {
        let source = r#"---
name: override
format: png
scale: 1
---

scale: 4
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        // Body value (parsed second) overrides frontmatter
        assert_eq!(target.scale, Some(4));
    }

    #[test]
    fn test_parse_target_with_comments() {
        let source = r#"---
name: commented
format: png
---

# Output settings
scale: 2
// padding not needed
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        assert_eq!(target.scale, Some(2));
        assert_eq!(target.padding, None);
    }

    #[test]
    fn test_parse_target_unknown_keys_ignored() {
        let source = r#"---
name: future
format: png
---

scale: 2
tile: 16x16
colors: 256
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        assert_eq!(target.scale, Some(2));
    }

    #[test]
    fn test_parse_multiple_targets() {
        let source = r#"---
name: web
format: png
---

---
name: sheet
format: png
---

sheet: auto
"#;

        let builders = parse_target_file(source).unwrap();
        assert_eq!(builders.len(), 2);

        let web = builders[0].clone().build().unwrap();
        assert_eq!(web.name, "web");

        let sheet = builders[1].clone().build().unwrap();
        assert_eq!(sheet.name, "sheet");
        assert_eq!(sheet.sheet, SheetConfig::Auto);
    }

    #[test]
    fn test_parse_target_palette_mode() {
        let source = r#"---
name: indexed-target
format: png
---

palette_mode: indexed
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        assert_eq!(target.palette_mode, crate::types::PaletteMode::Indexed);
    }

    #[test]
    fn test_parse_target_default_format() {
        let source = r#"---
name: no-format
---
"#;

        let builders = parse_target_file(source).unwrap();
        let target = builders[0].clone().build().unwrap();

        assert_eq!(target.format, "png");
    }
}
