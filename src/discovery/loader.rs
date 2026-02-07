//! Asset loader - parses discovered files into registry.
//!
//! Takes scan results and loads all assets into a RegistryBuilder.

use std::fs;
use std::path::Path;

use crate::error::{PxError, Result};
use crate::parser::{parse_brush_file, parse_map_file, parse_palette, parse_prefab_file, parse_shader_file, parse_shape_file, parse_stamp_file};
use crate::registry::RegistryBuilder;
use crate::types::{BuiltinBrushes, BuiltinStamps, Palette};

use super::scanner::ScanResult;

/// Options for loading assets.
#[derive(Debug, Clone, Default)]
pub struct LoadOptions {
    /// Include builtin stamps (corner, edge-h, etc.).
    pub include_builtin_stamps: bool,
    /// Include builtin brushes (solid, checker, etc.).
    pub include_builtin_brushes: bool,
    /// Include default palette if no palettes found.
    pub include_default_palette: bool,
}

impl LoadOptions {
    /// Create options with all builtins included.
    pub fn with_builtins() -> Self {
        Self {
            include_builtin_stamps: true,
            include_builtin_brushes: true,
            include_default_palette: true,
        }
    }
}

/// Load assets from scan result into a RegistryBuilder.
///
/// Parses all discovered files and adds them to the builder.
/// Returns errors for any files that fail to parse.
pub fn load_assets(scan: &ScanResult, options: &LoadOptions) -> Result<RegistryBuilder> {
    let mut builder = RegistryBuilder::new();
    let mut errors: Vec<String> = Vec::new();

    // Load palettes
    for path in &scan.palettes {
        match load_palette(path) {
            Ok(palettes) => {
                builder.add_palettes(palettes);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Load stamps
    for path in &scan.stamps {
        match load_stamps(path) {
            Ok(stamps) => {
                builder.add_stamps(stamps);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Load brushes
    for path in &scan.brushes {
        match load_brushes(path) {
            Ok(brushes) => {
                builder.add_brushes(brushes);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Load shaders
    for path in &scan.shaders {
        match load_shaders(path) {
            Ok(shaders) => {
                builder.add_shaders(shaders);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Load shapes
    for path in &scan.shapes {
        match load_shapes(path) {
            Ok(shapes) => {
                builder.add_shapes(shapes);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Load prefabs
    for path in &scan.prefabs {
        match load_prefabs(path) {
            Ok(prefabs) => {
                builder.add_prefabs(prefabs);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Load maps
    for path in &scan.maps {
        match load_maps(path) {
            Ok(maps) => {
                builder.add_maps(maps);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Add builtins if requested
    if options.include_builtin_stamps {
        builder.add_stamps(BuiltinStamps::all());
    }

    if options.include_builtin_brushes {
        builder.add_brushes(BuiltinBrushes::all());
    }

    if options.include_default_palette && scan.palettes.is_empty() {
        builder.add_palette(Palette::default_palette());
    }

    // Report errors if any
    if !errors.is_empty() {
        return Err(PxError::Build {
            message: format!("Failed to load {} file(s):\n  {}", errors.len(), errors.join("\n  ")),
            help: Some("Fix the errors above and try again".to_string()),
        });
    }

    Ok(builder)
}

/// Load palettes from a file.
fn load_palette(path: &Path) -> Result<Vec<Palette>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    let builders = parse_palette(&content)?;
    builders
        .into_iter()
        .map(|b| {
            b.build(None).map_err(|e| PxError::Build {
                message: e.to_string(),
                help: None,
            })
        })
        .collect()
}

/// Load stamps from a file.
fn load_stamps(path: &Path) -> Result<Vec<crate::types::Stamp>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    parse_stamp_file(&content)
}

/// Load brushes from a file.
fn load_brushes(path: &Path) -> Result<Vec<crate::types::Brush>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    parse_brush_file(&content)
}

/// Load shaders from a file.
fn load_shaders(path: &Path) -> Result<Vec<crate::types::Shader>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    let builders = parse_shader_file(&content)?;

    builders
        .into_iter()
        .map(|b| {
            b.build(None).map_err(|e| PxError::Build {
                message: e.to_string(),
                help: None,
            })
        })
        .collect()
}

/// Load shapes from a file.
fn load_shapes(path: &Path) -> Result<Vec<crate::types::Shape>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    parse_shape_file(&content)
}

/// Load prefabs from a file.
fn load_prefabs(path: &Path) -> Result<Vec<crate::types::Prefab>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    parse_prefab_file(&content)
}

/// Load maps from a file.
fn load_maps(path: &Path) -> Result<Vec<crate::types::Map>> {
    let content = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    parse_map_file(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_empty_scan() {
        let scan = ScanResult::default();
        let options = LoadOptions::default();

        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.is_empty());
    }

    #[test]
    fn test_load_with_builtins() {
        let scan = ScanResult::default();
        let options = LoadOptions::with_builtins();

        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        // Should have builtin stamps and brushes
        assert!(registry.get_stamp("corner").is_some());
        assert!(registry.get_brush("checker").is_some());
        assert!(registry.get_palette("default").is_some());
    }

    #[test]
    fn test_load_palette_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.palette.md");

        fs::write(
            &path,
            r#"---
name: test
---
$dark: #000000
$light: #ffffff
"#,
        )
        .unwrap();

        let mut scan = ScanResult::default();
        scan.palettes.push(path);

        let options = LoadOptions::default();
        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.get_palette("test").is_some());
    }

    #[test]
    fn test_load_shape_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.shape.md");

        fs::write(
            &path,
            r#"---
name: test-shape
---

```px
##
##
```
"#,
        )
        .unwrap();

        let mut scan = ScanResult::default();
        scan.shapes.push(path);

        let options = LoadOptions::default();
        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.get_shape("test-shape").is_some());
    }

    #[test]
    fn test_load_multiple_shapes_from_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("multi.shape.md");

        fs::write(
            &path,
            r#"---
name: shape-a
---

```px
#
```

---
name: shape-b
---

```px
.
```
"#,
        )
        .unwrap();

        let mut scan = ScanResult::default();
        scan.shapes.push(path);

        let options = LoadOptions::default();
        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.get_shape("shape-a").is_some());
        assert!(registry.get_shape("shape-b").is_some());
    }

    #[test]
    fn test_load_invalid_file_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.shape.md");

        fs::write(&path, "this is not valid yaml frontmatter").unwrap();

        let mut scan = ScanResult::default();
        scan.shapes.push(path);

        let options = LoadOptions::default();
        let result = load_assets(&scan, &options);

        assert!(result.is_err());
    }

    #[test]
    fn test_load_brush_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.brush.md");

        fs::write(
            &path,
            r#"---
name: test-brush
---

```px
AB
BA
```
"#,
        )
        .unwrap();

        let mut scan = ScanResult::default();
        scan.brushes.push(path);

        let options = LoadOptions::default();
        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.get_brush("test-brush").is_some());
    }

    #[test]
    fn test_load_stamp_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.stamp.md");

        fs::write(
            &path,
            r#"---
name: test-stamp
glyph: T
---

```px
$
```
"#,
        )
        .unwrap();

        let mut scan = ScanResult::default();
        scan.stamps.push(path);

        let options = LoadOptions::default();
        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.get_stamp("test-stamp").is_some());
    }

    #[test]
    fn test_load_shader_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.shader.md");

        fs::write(
            &path,
            r#"---
name: test-shader
palette: default
---
"#,
        )
        .unwrap();

        let mut scan = ScanResult::default();
        scan.shaders.push(path);

        let options = LoadOptions::default();
        let builder = load_assets(&scan, &options).unwrap();
        let registry = builder.build().unwrap();

        assert!(registry.get_shader("test-shader").is_some());
    }
}
