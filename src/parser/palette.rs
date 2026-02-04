//! Palette file parser.
//!
//! Parses `.palette.md` files into `PaletteBuilder` instances.

use crate::error::{PxError, Result};
use crate::parser::{parse_documents, RawDocument};
use crate::types::PaletteBuilder;

/// Parse a palette file into one or more palette builders.
///
/// Each document in the file becomes a separate palette builder.
/// The builders must be resolved (with `build()`) to get final palettes.
pub fn parse_palette_file(source: &str) -> Result<Vec<PaletteBuilder>> {
    let documents = parse_documents(source)?;

    documents.into_iter().map(parse_palette_document).collect()
}

/// Parse a single palette document into a builder.
fn parse_palette_document(doc: RawDocument) -> Result<PaletteBuilder> {
    let mut builder = PaletteBuilder::new(doc.name.value.clone());

    // Check for inheritance
    if let Some(inherits) = doc.get_frontmatter_str("inherits") {
        builder.inherits(inherits);
    }

    // Parse the body for colour definitions
    // Palette files don't use ```px blocks, they have definitions after frontmatter
    // The "body" in this case is everything after the frontmatter, not in a code fence

    // For palettes, we need to look at what comes after the frontmatter.
    // The RawDocument.body only captures content in ```px blocks, which palettes don't use.
    // Instead, palette definitions are in the raw text after frontmatter.
    //
    // We need to re-parse the source to get the palette body.
    // For now, let's handle this by parsing the frontmatter values that look like colour defs.

    // Actually, looking at the spec and example more carefully:
    // The palette body is NOT in a code fence - it's just lines after ---
    // So we need a different approach: parse the raw content after frontmatter

    // Since RawDocument doesn't give us access to raw content after frontmatter
    // (it only extracts ```px blocks), we need to handle palette files specially.
    // Let's re-parse the source directly for palette-specific content.

    Err(PxError::Parse {
        message: "Palette parsing requires special handling".to_string(),
        help: Some("This is a placeholder - implementing proper palette body parsing".to_string()),
    })
}

/// Parse palette content (the body section after frontmatter).
///
/// This parses lines like:
/// - `$name: #hex`
/// - `$name: $reference`
/// - `@variant-name:` blocks
pub fn parse_palette_content(content: &str, builder: &mut PaletteBuilder) -> Result<()> {
    let mut current_variant: Option<String> = None;
    let mut in_variant_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Check for variant block start: @variant-name:
        if trimmed.starts_with('@') {
            if let Some(variant_name) = parse_variant_header(trimmed) {
                current_variant = Some(variant_name);
                in_variant_block = true;
                continue;
            }
        }

        // Check for colour definition: $name: value
        if trimmed.starts_with('$') {
            if let Some((name, value)) = parse_colour_line(trimmed)? {
                if in_variant_block {
                    if let Some(ref variant) = current_variant {
                        builder.define_variant(variant, name, value);
                    }
                } else {
                    builder.define(name, value);
                }
            }
            continue;
        }

        // Check for end of variant block (unindented non-@ line that's not a colour)
        if in_variant_block && !line.starts_with(' ') && !line.starts_with('\t') {
            // End of variant block if it's not a colour def or variant header
            if !trimmed.starts_with('$') && !trimmed.starts_with('@') {
                in_variant_block = false;
                current_variant = None;
            }
        }

        // Indented lines in a variant block
        if in_variant_block && (line.starts_with(' ') || line.starts_with('\t')) {
            if trimmed.starts_with('$') {
                if let Some((name, value)) = parse_colour_line(trimmed)? {
                    if let Some(ref variant) = current_variant {
                        builder.define_variant(variant, name, value);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Parse a variant header line: `@variant-name:`
fn parse_variant_header(line: &str) -> Option<String> {
    let line = line.strip_prefix('@')?;
    let line = line.strip_suffix(':')?;
    Some(line.trim().to_string())
}

/// Parse a colour definition line: `$name: value`
fn parse_colour_line(line: &str) -> Result<Option<(String, String)>> {
    let line = line.strip_prefix('$').unwrap_or(line);

    let colon_pos = match line.find(':') {
        Some(pos) => pos,
        None => return Ok(None),
    };

    let name = line[..colon_pos].trim().to_string();
    let value = line[colon_pos + 1..].trim().to_string();

    if name.is_empty() {
        return Err(PxError::Parse {
            message: "Empty colour name".to_string(),
            help: None,
        });
    }

    if value.is_empty() {
        return Err(PxError::Parse {
            message: format!("Empty colour value for ${}", name),
            help: None,
        });
    }

    Ok(Some((name, value)))
}

/// Parse a complete palette from source, handling the special palette format.
///
/// Unlike other file types, palettes don't use ```px blocks.
/// The colour definitions are directly after the frontmatter.
pub fn parse_palette(source: &str) -> Result<Vec<PaletteBuilder>> {
    // Split the source into sections for multi-document files
    let sections = split_palette_sections(source);

    let mut builders = Vec::new();

    for section in sections {
        let builder = parse_single_palette(&section)?;
        builders.push(builder);
    }

    if builders.is_empty() {
        return Err(PxError::Parse {
            message: "No palette definitions found".to_string(),
            help: Some("Add a palette with ---\\nname: my-palette\\n---".to_string()),
        });
    }

    Ok(builders)
}

/// Split source into palette sections (for multi-palette files).
///
/// Palette files can contain multiple palettes, each starting with `---`
/// followed by frontmatter containing `name:`. We split on `---` that is
/// followed by a line containing `name:`.
fn split_palette_sections(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut sections = Vec::new();
    let mut current_start = 0;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Look for --- that starts a new document (not the closing --- of frontmatter)
        // A new document has --- followed by a line with name:
        if line == "---" && i > 0 {
            // Check if this looks like a new document (next line has name:)
            if let Some(next_line) = lines.get(i + 1) {
                if next_line.trim().starts_with("name:") {
                    // This is a new document, save current section
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

/// Parse a single palette section.
fn parse_single_palette(source: &str) -> Result<PaletteBuilder> {
    let trimmed = source.trim();

    // Must start with ---
    if !trimmed.starts_with("---") {
        return Err(PxError::Parse {
            message: "Palette must start with ---".to_string(),
            help: Some("Add YAML frontmatter: ---\\nname: my-palette\\n---".to_string()),
        });
    }

    // Find closing --- (must be on its own line)
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(after_opening);
    let after_opening = after_opening.strip_prefix('\r').unwrap_or(after_opening);

    // Look for --- at the start of a line
    let mut closing_pos = None;
    let mut pos = 0;
    for line in after_opening.lines() {
        if line.trim() == "---" {
            closing_pos = Some(pos);
            break;
        }
        pos += line.len() + 1; // +1 for newline
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
            message: "Palette missing required 'name' field".to_string(),
            help: None,
        })?;

    let mut builder = PaletteBuilder::new(name);

    // Check for inheritance
    if let Some(inherits) = frontmatter.get("inherits").and_then(|v| v.as_str()) {
        builder.inherits(inherits);
    }

    // Parse colour definitions from body
    parse_palette_content(body, &mut builder)?;

    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Colour;

    #[test]
    fn test_parse_simple_palette() {
        let source = "---\nname: test\n---\n$dark: #1a1a2e\n$light: #4a4a68\n";

        let builders = parse_palette(source).unwrap();
        assert_eq!(builders.len(), 1);

        let palette = builders.into_iter().next().unwrap().build(None).unwrap();
        assert_eq!(palette.name, "test");
        assert_eq!(palette.get("dark"), Some(Colour::from_hex("#1a1a2e").unwrap()));
        assert_eq!(palette.get("light"), Some(Colour::from_hex("#4a4a68").unwrap()));
    }

    #[test]
    fn test_parse_palette_with_references() {
        let source = r#"---
name: test
---
$dark: #1a1a2e
$edge: $dark
"#;

        let builders = parse_palette(source).unwrap();
        let palette = builders.into_iter().next().unwrap().build(None).unwrap();

        assert_eq!(palette.get("edge"), palette.get("dark"));
    }

    #[test]
    fn test_parse_palette_with_variant() {
        let source = r#"---
name: test
---
$dark: #000000

@light-mode:
  $dark: #FFFFFF
"#;

        let builders = parse_palette(source).unwrap();
        let palette = builders.into_iter().next().unwrap().build(None).unwrap();

        assert_eq!(palette.get("dark"), Some(Colour::BLACK));
        assert_eq!(
            palette.get_with_variant("dark", "light-mode"),
            Some(Colour::WHITE)
        );
    }

    #[test]
    fn test_parse_real_example() {
        let source = r#"---
name: mi
scheme: retro
---

$gold: #F7AD45
$green-dark: #657C6A
$green-light: #A2B9A7

$black: #1a1a1a
$white: #f0f0f0

$edge: $black
$fill: $green-dark
$highlight: $gold
$background: $green-light
"#;

        let builders = parse_palette(source).unwrap();
        let palette = builders.into_iter().next().unwrap().build(None).unwrap();

        assert_eq!(palette.name, "mi");
        assert_eq!(palette.get("gold"), Some(Colour::from_hex("#F7AD45").unwrap()));
        assert_eq!(palette.get("edge"), palette.get("black"));
        assert_eq!(palette.get("fill"), palette.get("green-dark"));
    }

    #[test]
    fn test_parse_palette_with_inheritance() {
        let parent_source = r#"---
name: parent
---
$base: #FF0000
$edge: $base
"#;

        let child_source = r#"---
name: child
inherits: parent
---
$custom: #00FF00
"#;

        let parent_builders = parse_palette(parent_source).unwrap();
        let parent = parent_builders
            .into_iter()
            .next()
            .unwrap()
            .build(None)
            .unwrap();

        let child_builders = parse_palette(child_source).unwrap();
        let child_builder = child_builders.into_iter().next().unwrap();

        assert_eq!(child_builder.parent_name(), Some("parent"));

        let child = child_builder.build(Some(&parent)).unwrap();

        // Inherited
        assert_eq!(child.get("base"), Some(Colour::rgb(255, 0, 0)));
        assert_eq!(child.get("edge"), Some(Colour::rgb(255, 0, 0)));
        // New
        assert_eq!(child.get("custom"), Some(Colour::rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_colour_line() {
        let result = parse_colour_line("$dark: #1a1a2e").unwrap();
        assert_eq!(result, Some(("dark".to_string(), "#1a1a2e".to_string())));

        let result = parse_colour_line("$edge: $dark").unwrap();
        assert_eq!(result, Some(("edge".to_string(), "$dark".to_string())));
    }

    #[test]
    fn test_parse_variant_header() {
        assert_eq!(
            parse_variant_header("@light-mode:"),
            Some("light-mode".to_string())
        );
        assert_eq!(parse_variant_header("@dark:"), Some("dark".to_string()));
        assert_eq!(parse_variant_header("not-a-variant"), None);
    }

    #[test]
    fn test_missing_name() {
        let source = r#"---
scheme: test
---
$dark: #000000
"#;

        let result = parse_palette(source);
        assert!(result.is_err());
    }
}
