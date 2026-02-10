//! Build command implementation.
//!
//! Processes shape files and outputs PNG images.

use std::fs;
use std::path::PathBuf;

use clap::Args;

use crate::discovery::{discover_paths, LoadOptions};
use crate::error::{PxError, Result};
use crate::parser::{parse_map_file, parse_prefab_file, parse_shape_file, parse_shader_file, parse_target_file};
use crate::render::{write_png, write_sheet_json, MapRenderer, PrefabRenderer, RenderedShape, ShapeRenderer, SheetPacker};
use crate::types::{BuiltinBrushes, BuiltinShaders, BuiltinStamps, BuiltinTargets, Palette, Shader, ShapeMetadata, SheetConfig, Target};
use crate::validation::{print_diagnostics, validate_registry};

/// Build sprites and maps from definition files
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Input files to process
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Shader to apply
    #[arg(long)]
    pub shader: Option<String>,

    /// Output target profile
    #[arg(long)]
    pub target: Option<String>,

    /// Output directory
    #[arg(long, short, default_value = "dist")]
    pub output: PathBuf,

    /// Scale factor for output (integer upscaling)
    #[arg(long)]
    pub scale: Option<u32>,

    /// Run validation checks before building
    #[arg(long)]
    pub validate: bool,

    /// Pack all sprites into a single sprite sheet
    #[arg(long)]
    pub sheet: bool,

    /// Padding between sprites in sheet (pixels)
    #[arg(long)]
    pub padding: Option<u32>,
}

pub fn run(args: BuildArgs) -> Result<()> {
    // Run validation if requested
    if args.validate {
        let discovery = discover_paths(&args.files)?;
        let builder =
            crate::discovery::load_assets(&discovery.scan, &LoadOptions::with_builtins())?;
        let registry = builder.build()?;
        let result = validate_registry(&registry);
        print_diagnostics(&result);

        if result.has_errors() {
            return Err(PxError::Build {
                message: "Validation failed, aborting build".to_string(),
                help: Some("Fix the errors above and try again".to_string()),
            });
        }
    }

    // Create output directory if needed
    if !args.output.exists() {
        fs::create_dir_all(&args.output).map_err(|e| PxError::Io {
            path: args.output.clone(),
            message: format!("Failed to create output directory: {}", e),
        })?;
    }

    // Resolve target profile (if specified)
    let target = resolve_target(&args)?;

    // Compute effective settings: CLI > target > defaults
    let effective_scale = args
        .scale
        .or_else(|| target.as_ref().and_then(|t| t.scale));
    let effective_padding = args
        .padding
        .or_else(|| target.as_ref().and_then(|t| t.padding))
        .unwrap_or(0);
    let effective_sheet = if args.sheet {
        SheetConfig::Auto
    } else {
        target
            .as_ref()
            .map(|t| t.sheet.clone())
            .unwrap_or(SheetConfig::None)
    };
    let effective_shader_name = args
        .shader
        .clone()
        .or_else(|| target.as_ref().and_then(|t| t.shader.clone()));

    // Load shader (use builtin default if not specified)
    let shader = load_shader_by_name(effective_shader_name.as_deref())?;

    // Get palette from shader
    let palette = load_palette_for_shader(&shader)?;

    // Collect builtin stamps and brushes (need to own them for lifetime)
    let builtin_stamps = BuiltinStamps::all();
    let builtin_brushes = BuiltinBrushes::all();

    // Create renderer
    let mut renderer = ShapeRenderer::new(&palette);

    // Add builtin stamps
    for stamp in &builtin_stamps {
        renderer.add_stamp(stamp);
    }

    // Add builtin brushes
    for brush in &builtin_brushes {
        renderer.add_brush(brush);
    }

    // Set palette variant if specified in shader
    let renderer = if let Some(variant) = &shader.palette_variant {
        renderer.with_variant(variant)
    } else {
        renderer
    };

    let use_sheet = effective_sheet != SheetConfig::None;
    let write_individual = !use_sheet;

    // Phase 1: Render shapes
    let mut total_shapes = 0;
    let mut rendered_shapes: Vec<RenderedShape> = Vec::new();
    let mut prefab_files: Vec<PathBuf> = Vec::new();
    let mut map_files: Vec<PathBuf> = Vec::new();

    for file in &args.files {
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "md" => {
                let filename = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if filename.contains(".shape.") {
                    let (count, rendered) =
                        process_shape_file(file, &args.output, effective_scale, &renderer, write_individual)?;
                    total_shapes += count;
                    rendered_shapes.extend(rendered);
                } else if filename.contains(".prefab.") {
                    prefab_files.push(file.clone());
                } else if filename.contains(".map.") {
                    map_files.push(file.clone());
                } else {
                    eprintln!("Skipping unsupported file: {}", file.display());
                }
            }
            _ => {
                eprintln!("Skipping unsupported file: {}", file.display());
            }
        }
    }

    // Phase 2: Render prefabs (need rendered shapes)
    let mut total_prefabs = 0;
    let mut rendered_prefabs: Vec<RenderedShape> = Vec::new();
    if !prefab_files.is_empty() {
        let mut prefab_renderer = PrefabRenderer::new();
        for shape in &rendered_shapes {
            prefab_renderer.add_rendered(shape.clone());
        }

        for file in &prefab_files {
            let (count, rendered) =
                process_prefab_file(file, &args.output, effective_scale, &mut prefab_renderer, write_individual)?;
            total_prefabs += count;
            rendered_prefabs.extend(rendered);
        }
    }

    // Phase 3: Render maps (skip when packing a sheet)
    let mut total_maps = 0;
    if !use_sheet && !map_files.is_empty() {
        let mut map_renderer = MapRenderer::new();
        for shape in &rendered_shapes {
            map_renderer.add_rendered(shape.clone());
        }
        for prefab in &rendered_prefabs {
            map_renderer.add_rendered(prefab.clone());
        }

        for file in &map_files {
            total_maps += process_map_file(file, &args.output, effective_scale, &map_renderer)?;
        }
    }

    // Sheet packing mode: combine all sprites into one sheet
    if use_sheet {
        let mut all_sprites: Vec<RenderedShape> = Vec::new();
        all_sprites.extend(rendered_shapes.iter().cloned());
        all_sprites.extend(rendered_prefabs.iter().cloned());

        let packer = SheetPacker::new(effective_padding);
        let (sheet, meta) = packer.pack(&all_sprites);

        let png_path = args.output.join("sheet.png");
        let json_path = args.output.join("sheet.json");

        let sheet_scale = effective_scale.unwrap_or(1);
        write_png(&sheet, &png_path, sheet_scale)?;
        write_sheet_json(&meta, &json_path)?;

        let total = total_shapes + total_prefabs;
        println!(
            "Packed {} sprite(s) into {} + {}",
            total,
            png_path.display(),
            json_path.display()
        );
    } else {
        let total = total_shapes + total_prefabs + total_maps;
        println!("Built {} asset(s) to {}", total, args.output.display());
    }

    Ok(())
}

/// Process a shape file and render shapes.
/// When `write_png_files` is true, writes individual PNGs.
/// Returns the count and the rendered shapes (for prefab compositing or sheet packing).
fn process_shape_file(
    path: &PathBuf,
    output: &PathBuf,
    default_scale: Option<u32>,
    renderer: &ShapeRenderer,
    write_png_files: bool,
) -> Result<(usize, Vec<RenderedShape>)> {
    let source = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.clone(),
        message: format!("Failed to read file: {}", e),
    })?;

    let shapes = parse_shape_file(&source)?;
    let mut rendered_shapes = Vec::new();

    for shape in &shapes {
        let scale = if let Some(s) = default_scale {
            if s > 1 { s } else { shape.scale.unwrap_or(1) }
        } else {
            shape.scale.unwrap_or(1)
        };

        let rendered = renderer.render(shape);

        if write_png_files {
            let png_name = format!("{}.png", shape.name);
            let png_path = output.join(&png_name);
            write_png(&rendered, &png_path, scale)?;

            // Write JSON metadata
            let metadata = ShapeMetadata {
                name: shape.name.clone(),
                size: [rendered.width(), rendered.height()],
                tags: shape.tags.clone(),
            };
            let json_name = format!("{}.json", shape.name);
            let json_path = output.join(&json_name);
            write_metadata_json(&metadata, &json_path)?;

            println!("  {} -> {} + {}", shape.name, png_path.display(), json_path.display());
        }

        rendered_shapes.push(rendered);
    }

    Ok((shapes.len(), rendered_shapes))
}

/// Process a prefab file and render prefabs.
/// When `write_png_files` is true, writes individual PNGs.
/// Returns the count and the rendered prefabs (for map compositing or sheet packing).
/// Rendered prefabs are also added to the renderer for nested prefab support.
fn process_prefab_file(
    path: &PathBuf,
    output: &PathBuf,
    default_scale: Option<u32>,
    prefab_renderer: &mut PrefabRenderer,
    write_png_files: bool,
) -> Result<(usize, Vec<RenderedShape>)> {
    let source = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.clone(),
        message: format!("Failed to read file: {}", e),
    })?;

    let prefabs = parse_prefab_file(&source)?;
    let mut rendered_prefabs = Vec::new();

    for prefab in &prefabs {
        let scale = if let Some(s) = default_scale {
            if s > 1 { s } else { prefab.scale.unwrap_or(1) }
        } else {
            prefab.scale.unwrap_or(1)
        };

        let (rendered, metadata) = prefab_renderer.render(prefab)?;

        if write_png_files {
            let png_name = format!("{}.png", prefab.name);
            let png_path = output.join(&png_name);
            write_png(&rendered, &png_path, scale)?;

            // Write JSON metadata
            let json_name = format!("{}.json", prefab.name);
            let json_path = output.join(&json_name);
            write_metadata_json(&metadata, &json_path)?;

            println!("  {} -> {} + {}", prefab.name, png_path.display(), json_path.display());
        }

        // Add rendered prefab so later prefabs can reference it
        prefab_renderer.add_rendered(rendered.clone());
        rendered_prefabs.push(rendered);
    }

    Ok((prefabs.len(), rendered_prefabs))
}

/// Process a map file and write PNG + JSON output.
fn process_map_file(
    path: &PathBuf,
    output: &PathBuf,
    default_scale: Option<u32>,
    map_renderer: &MapRenderer,
) -> Result<usize> {
    let source = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.clone(),
        message: format!("Failed to read file: {}", e),
    })?;

    let maps = parse_map_file(&source)?;

    for map in &maps {
        let scale = if let Some(s) = default_scale {
            if s > 1 { s } else { map.scale.unwrap_or(1) }
        } else {
            map.scale.unwrap_or(1)
        };

        let (rendered, metadata) = map_renderer.render(map)?;

        // Write PNG
        let png_name = format!("{}.png", map.name);
        let png_path = output.join(&png_name);
        write_png(&rendered, &png_path, scale)?;

        // Write JSON metadata
        let json_name = format!("{}.json", map.name);
        let json_path = output.join(&json_name);
        write_metadata_json(&metadata, &json_path)?;

        println!("  {} -> {} + {}", map.name, png_path.display(), json_path.display());
    }

    Ok(maps.len())
}

/// Write a serializable metadata value as JSON to a file.
fn write_metadata_json(value: &impl serde::Serialize, path: &std::path::Path) -> Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(|e| PxError::Build {
        message: format!("Failed to serialize metadata: {}", e),
        help: None,
    })?;
    fs::write(path, json).map_err(|e| PxError::Io {
        path: path.to_path_buf(),
        message: format!("Failed to write metadata: {}", e),
    })?;
    Ok(())
}

/// Resolve target from CLI args: check builtins, then try file path.
fn resolve_target(args: &BuildArgs) -> Result<Option<Target>> {
    let target_name = match &args.target {
        Some(name) => name,
        None => return Ok(None),
    };

    // Check builtins first
    if let Some(target) = BuiltinTargets::get(target_name) {
        return Ok(Some(target));
    }

    // Try as a file path
    let target_path = PathBuf::from(target_name);
    if target_path.exists() {
        let source = fs::read_to_string(&target_path).map_err(|e| PxError::Io {
            path: target_path.clone(),
            message: format!("Failed to read target file: {}", e),
        })?;

        let builders = parse_target_file(&source)?;
        let builder = builders.into_iter().next().ok_or_else(|| PxError::Parse {
            message: format!("No targets found in {}", target_path.display()),
            help: None,
        })?;

        let target = builder.build().map_err(|e| PxError::Build {
            message: format!("Failed to build target: {}", e),
            help: None,
        })?;

        return Ok(Some(target));
    }

    Err(PxError::Build {
        message: format!("Target not found: {}", target_name),
        help: Some("Use 'web' or 'sheet' for builtin targets, or provide a .target.md file path".to_string()),
    })
}

/// Load shader by name or use builtin default.
fn load_shader_by_name(name: Option<&str>) -> Result<Shader> {
    let shader_name = match name {
        Some(n) => n,
        None => return Ok(BuiltinShaders::get("default").unwrap()),
    };

    // Check if it's a builtin
    if let Some(shader) = BuiltinShaders::get(shader_name) {
        return Ok(shader);
    }

    // Try to find shader file
    let shader_path = PathBuf::from(shader_name);
    if shader_path.exists() {
        let source = fs::read_to_string(&shader_path).map_err(|e| PxError::Io {
            path: shader_path.clone(),
            message: format!("Failed to read shader file: {}", e),
        })?;

        let builders = parse_shader_file(&source)?;
        let builder = builders.into_iter().next().ok_or_else(|| PxError::Parse {
            message: format!("No shaders found in {}", shader_path.display()),
            help: None,
        })?;

        return builder.build(None).map_err(|e| PxError::Build {
            message: format!("Failed to build shader: {}", e),
            help: None,
        });
    }

    Err(PxError::Build {
        message: format!("Shader not found: {}", shader_name),
        help: Some("Use 'default' for the builtin shader or provide a path".to_string()),
    })
}

/// Load palette for a shader.
fn load_palette_for_shader(shader: &Shader) -> Result<Palette> {
    // For now, just use the default palette
    // In Phase 2+, this will resolve palette by name
    if shader.palette == "default" {
        return Ok(Palette::default_palette());
    }

    // TODO: Implement palette file loading
    // For now, fall back to default
    Ok(Palette::default_palette())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_simple_shape() {
        let dir = tempdir().unwrap();
        let shape_path = dir.path().join("test.shape.md");
        let output_dir = dir.path().join("output");

        // Write a simple shape file
        fs::write(
            &shape_path,
            r#"---
name: test-square
---

```px
+--+
|..|
|..|
+--+
```
"#,
        )
        .unwrap();

        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: None,
            output: output_dir.clone(),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        // Check output exists
        let output_png = output_dir.join("test-square.png");
        assert!(output_png.exists());

        // Verify dimensions
        let img = image::open(&output_png).unwrap().to_rgba8();
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
    }

    #[test]
    fn test_build_with_scale() {
        let dir = tempdir().unwrap();
        let shape_path = dir.path().join("scaled.shape.md");
        let output_dir = dir.path().join("output");

        fs::write(
            &shape_path,
            r#"---
name: scaled-shape
---

```px
##
##
```
"#,
        )
        .unwrap();

        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: None,
            output: output_dir.clone(),
            scale: Some(4),
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        let output_png = output_dir.join("scaled-shape.png");
        let img = image::open(&output_png).unwrap().to_rgba8();

        // 2x2 shape scaled 4x = 8x8
        assert_eq!(img.width(), 8);
        assert_eq!(img.height(), 8);
    }

    #[test]
    fn test_build_multiple_shapes_in_file() {
        let dir = tempdir().unwrap();
        let shape_path = dir.path().join("multi.shape.md");
        let output_dir = dir.path().join("output");

        fs::write(
            &shape_path,
            r#"---
name: shape-a
---

```px
##
```

---
name: shape-b
---

```px
..
```
"#,
        )
        .unwrap();

        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: None,
            output: output_dir.clone(),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        assert!(output_dir.join("shape-a.png").exists());
        assert!(output_dir.join("shape-b.png").exists());
    }

    #[test]
    fn test_build_with_frontmatter_scale() {
        let dir = tempdir().unwrap();
        let shape_path = dir.path().join("frontmatter-scale.shape.md");
        let output_dir = dir.path().join("output");

        // Shape with scale in frontmatter
        fs::write(
            &shape_path,
            r#"---
name: fm-scaled
scale: 2
---

```px
#.
.#
```
"#,
        )
        .unwrap();

        // CLI scale is None (default), so frontmatter scale should be used
        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: None,
            output: output_dir.clone(),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        let output_png = output_dir.join("fm-scaled.png");
        let img = image::open(&output_png).unwrap().to_rgba8();

        // 2x2 shape scaled 2x from frontmatter = 4x4
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
    }

    #[test]
    fn test_build_cli_scale_overrides_frontmatter() {
        let dir = tempdir().unwrap();
        let shape_path = dir.path().join("override-scale.shape.md");
        let output_dir = dir.path().join("output");

        // Shape with scale: 2 in frontmatter
        fs::write(
            &shape_path,
            r#"---
name: override-test
scale: 2
---

```px
##
##
```
"#,
        )
        .unwrap();

        // CLI scale is 4, should override frontmatter's 2
        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: None,
            output: output_dir.clone(),
            scale: Some(4),
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        let output_png = output_dir.join("override-test.png");
        let img = image::open(&output_png).unwrap().to_rgba8();

        // 2x2 shape scaled 4x from CLI (overrides frontmatter's 2) = 8x8
        assert_eq!(img.width(), 8);
        assert_eq!(img.height(), 8);
    }

    #[test]
    fn test_resolve_builtin_target_web() {
        let args = BuildArgs {
            files: vec![],
            shader: None,
            target: Some("web".to_string()),
            output: PathBuf::from("dist"),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        let target = resolve_target(&args).unwrap().unwrap();
        assert_eq!(target.name, "web");
        assert_eq!(target.format, "png");
        assert_eq!(target.sheet, SheetConfig::None);
    }

    #[test]
    fn test_resolve_builtin_target_sheet() {
        let args = BuildArgs {
            files: vec![],
            shader: None,
            target: Some("sheet".to_string()),
            output: PathBuf::from("dist"),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        let target = resolve_target(&args).unwrap().unwrap();
        assert_eq!(target.name, "sheet");
        assert_eq!(target.sheet, SheetConfig::Auto);
    }

    #[test]
    fn test_resolve_no_target() {
        let args = BuildArgs {
            files: vec![],
            shader: None,
            target: None,
            output: PathBuf::from("dist"),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        let target = resolve_target(&args).unwrap();
        assert!(target.is_none());
    }

    #[test]
    fn test_resolve_unknown_target() {
        let args = BuildArgs {
            files: vec![],
            shader: None,
            target: Some("pico8".to_string()),
            output: PathBuf::from("dist"),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        let result = resolve_target(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_target_file() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().join("custom.target.md");

        fs::write(
            &target_path,
            r#"---
name: custom
format: png
---

scale: 4
sheet: auto
padding: 2
"#,
        )
        .unwrap();

        let args = BuildArgs {
            files: vec![],
            shader: None,
            target: Some(target_path.to_string_lossy().to_string()),
            output: PathBuf::from("dist"),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        let target = resolve_target(&args).unwrap().unwrap();
        assert_eq!(target.name, "custom");
        assert_eq!(target.scale, Some(4));
        assert_eq!(target.sheet, SheetConfig::Auto);
        assert_eq!(target.padding, Some(2));
    }

    #[test]
    fn test_build_with_target_web() {
        let dir = tempdir().unwrap();
        let shape_path = dir.path().join("test.shape.md");
        let output_dir = dir.path().join("output");

        fs::write(
            &shape_path,
            r#"---
name: target-test
---

```px
##
##
```
"#,
        )
        .unwrap();

        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: Some("web".to_string()),
            output: output_dir.clone(),
            scale: None,
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        let output_png = output_dir.join("target-test.png");
        assert!(output_png.exists());
        let img = image::open(&output_png).unwrap().to_rgba8();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
    }

    #[test]
    fn test_cli_scale_overrides_target() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().join("scaled.target.md");
        let shape_path = dir.path().join("test.shape.md");
        let output_dir = dir.path().join("output");

        fs::write(
            &target_path,
            r#"---
name: scaled
format: png
---

scale: 2
"#,
        )
        .unwrap();

        fs::write(
            &shape_path,
            r#"---
name: override-target
---

```px
##
##
```
"#,
        )
        .unwrap();

        // CLI scale of 4 should override target's scale of 2
        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: Some(target_path.to_string_lossy().to_string()),
            output: output_dir.clone(),
            scale: Some(4),
            validate: false,
            sheet: false,
            padding: None,
        };

        run(args).unwrap();

        let output_png = output_dir.join("override-target.png");
        let img = image::open(&output_png).unwrap().to_rgba8();
        // 2x2 shape at scale 4 = 8x8
        assert_eq!(img.width(), 8);
        assert_eq!(img.height(), 8);
    }
}
