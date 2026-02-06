//! Build command implementation.
//!
//! Processes shape files and outputs PNG images.

use std::fs;
use std::path::PathBuf;

use clap::Args;

use crate::error::{PxError, Result};
use crate::parser::{parse_prefab_file, parse_shape_file, parse_shader_file};
use crate::render::{write_png, PrefabRenderer, RenderedShape, ShapeRenderer};
use crate::types::{BuiltinBrushes, BuiltinShaders, BuiltinStamps, Palette, Shader};

/// Build sprites and maps from definition files
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Input files to process
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Shader to apply
    #[arg(long)]
    pub shader: Option<String>,

    /// Output target
    #[arg(long)]
    pub target: Option<String>,

    /// Output directory
    #[arg(long, short, default_value = "dist")]
    pub output: PathBuf,

    /// Scale factor for output (integer upscaling)
    #[arg(long, default_value = "1")]
    pub scale: u32,
}

pub fn run(args: BuildArgs) -> Result<()> {
    // Create output directory if needed
    if !args.output.exists() {
        fs::create_dir_all(&args.output).map_err(|e| PxError::Io {
            path: args.output.clone(),
            message: format!("Failed to create output directory: {}", e),
        })?;
    }

    // Load shader (use builtin default if not specified)
    let shader = load_shader(&args)?;

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

    // Phase 1: Render shapes
    let mut total_shapes = 0;
    let mut rendered_shapes: Vec<RenderedShape> = Vec::new();
    let mut prefab_files: Vec<PathBuf> = Vec::new();

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
                    let (count, rendered) = process_shape_file(file, &args, &renderer)?;
                    total_shapes += count;
                    rendered_shapes.extend(rendered);
                } else if filename.contains(".prefab.") {
                    prefab_files.push(file.clone());
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
    if !prefab_files.is_empty() {
        let mut prefab_renderer = PrefabRenderer::new();
        for shape in &rendered_shapes {
            prefab_renderer.add_rendered(shape.clone());
        }

        for file in &prefab_files {
            total_prefabs += process_prefab_file(file, &args, &mut prefab_renderer)?;
        }
    }

    let total = total_shapes + total_prefabs;
    println!("Built {} asset(s) to {}", total, args.output.display());

    Ok(())
}

/// Process a shape file and write PNG output.
/// Returns the count and the rendered shapes (for prefab compositing).
fn process_shape_file(
    path: &PathBuf,
    args: &BuildArgs,
    renderer: &ShapeRenderer,
) -> Result<(usize, Vec<RenderedShape>)> {
    let source = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.clone(),
        message: format!("Failed to read file: {}", e),
    })?;

    let shapes = parse_shape_file(&source)?;
    let mut rendered_shapes = Vec::new();

    for shape in &shapes {
        let scale = if args.scale > 1 {
            args.scale
        } else {
            shape.scale.unwrap_or(1)
        };

        let rendered = renderer.render(shape);

        let output_name = format!("{}.png", shape.name);
        let output_path = args.output.join(&output_name);

        write_png(&rendered, &output_path, scale)?;
        println!("  {} -> {}", shape.name, output_path.display());

        rendered_shapes.push(rendered);
    }

    Ok((shapes.len(), rendered_shapes))
}

/// Process a prefab file and write PNG output.
/// Rendered prefabs are added to the renderer for nested prefab support.
fn process_prefab_file(
    path: &PathBuf,
    args: &BuildArgs,
    prefab_renderer: &mut PrefabRenderer,
) -> Result<usize> {
    let source = fs::read_to_string(path).map_err(|e| PxError::Io {
        path: path.clone(),
        message: format!("Failed to read file: {}", e),
    })?;

    let prefabs = parse_prefab_file(&source)?;

    for prefab in &prefabs {
        let scale = if args.scale > 1 {
            args.scale
        } else {
            prefab.scale.unwrap_or(1)
        };

        let rendered = prefab_renderer.render(prefab)?;

        let output_name = format!("{}.png", prefab.name);
        let output_path = args.output.join(&output_name);

        write_png(&rendered, &output_path, scale)?;
        println!("  {} -> {}", prefab.name, output_path.display());

        // Add rendered prefab so later prefabs can reference it
        prefab_renderer.add_rendered(rendered);
    }

    Ok(prefabs.len())
}

/// Load shader from file or use builtin default.
fn load_shader(args: &BuildArgs) -> Result<Shader> {
    if let Some(shader_name) = &args.shader {
        // Check if it's a builtin
        if let Some(shader) = BuiltinShaders::get(shader_name) {
            return Ok(shader);
        }

        // Try to find shader file
        // For now, assume shader name is a file path
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

        return Err(PxError::Build {
            message: format!("Shader not found: {}", shader_name),
            help: Some("Use 'default' for the builtin shader or provide a path".to_string()),
        });
    }

    // Default shader
    Ok(BuiltinShaders::get("default").unwrap())
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
            scale: 1,
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
            scale: 4,
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
            scale: 1,
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

        // CLI scale is 1 (default), so frontmatter scale should be used
        let args = BuildArgs {
            files: vec![shape_path],
            shader: None,
            target: None,
            output: output_dir.clone(),
            scale: 1,
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
            scale: 4,
        };

        run(args).unwrap();

        let output_png = output_dir.join("override-test.png");
        let img = image::open(&output_png).unwrap().to_rgba8();

        // 2x2 shape scaled 4x from CLI (overrides frontmatter's 2) = 8x8
        assert_eq!(img.width(), 8);
        assert_eq!(img.height(), 8);
    }
}
