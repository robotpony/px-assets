//! Init command implementation.
//!
//! Generates a `px.yaml` manifest from discovered assets.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use clap::Args;

use crate::discovery::{discover, MANIFEST_FILENAME};
use crate::error::{PxError, Result};
use crate::output::{display_path, plural, Printer};

/// Initialize a px project by generating a px.yaml manifest
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Directory to scan (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Overwrite existing px.yaml
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: InitArgs, printer: &Printer) -> Result<()> {
    let manifest_path = args.path.join(MANIFEST_FILENAME);

    // Check for existing manifest
    if manifest_path.exists() && !args.force {
        return Err(PxError::Build {
            message: format!("{} already exists", MANIFEST_FILENAME),
            help: Some("Use --force to overwrite".to_string()),
        });
    }

    // Discover assets (no manifest yet, so convention scanning)
    printer.status("Scanning", &display_path(&args.path));
    let discovery = discover(&args.path)?;
    let scan = &discovery.scan;

    // Collect unique parent directories (relative to project root)
    let mut source_dirs = BTreeSet::new();
    let all_files: Vec<&PathBuf> = scan
        .palettes
        .iter()
        .chain(&scan.stamps)
        .chain(&scan.brushes)
        .chain(&scan.shaders)
        .chain(&scan.shapes)
        .chain(&scan.prefabs)
        .chain(&scan.maps)
        .chain(&scan.targets)
        .collect();

    for file in &all_files {
        if let Some(parent) = file.parent() {
            // Make relative to project root
            let relative = parent
                .strip_prefix(&discovery.root)
                .unwrap_or(parent);

            let dir_str = if relative == std::path::Path::new("") {
                ".".to_string()
            } else {
                format!("{}/", relative.display())
            };
            source_dirs.insert(dir_str);
        }
    }

    // Build YAML manually for clean formatting
    let mut yaml = String::new();

    // Sources
    if source_dirs.is_empty() || (source_dirs.len() == 1 && source_dirs.contains(".")) {
        // Default: scan current directory, no need to list sources
    } else {
        yaml.push_str("sources:\n");
        for dir in &source_dirs {
            yaml.push_str(&format!("  - \"{}\"\n", dir));
        }
    }

    // Output
    yaml.push_str("output: dist\n");

    // Write manifest
    fs::write(&manifest_path, &yaml).map_err(|e| PxError::Io {
        path: manifest_path.clone(),
        message: format!("Failed to write manifest: {}", e),
    })?;

    let total = all_files.len();

    if !source_dirs.is_empty() {
        let dirs: Vec<&str> = source_dirs.iter().map(|s| s.as_str()).collect();
        printer.info("Discovered", &dirs.join(", "));
    }

    printer.success(
        "Created",
        &format!("{} ({} found)", MANIFEST_FILENAME, plural(total, "asset", "assets")),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Printer;
    use tempfile::tempdir;

    #[test]
    fn test_init_creates_manifest() {
        let dir = tempdir().unwrap();

        // Create a shape file so there's something to discover
        fs::write(
            dir.path().join("player.shape.md"),
            "---\nname: player\n---\n\n```px\n#\n```",
        )
        .unwrap();

        let args = InitArgs {
            path: dir.path().to_path_buf(),
            force: false,
        };

        run(args, &Printer::new()).unwrap();

        let manifest_path = dir.path().join("px.yaml");
        assert!(manifest_path.exists());

        let content = fs::read_to_string(&manifest_path).unwrap();
        assert!(content.contains("output: dist"));
    }

    #[test]
    fn test_init_errors_if_manifest_exists() {
        let dir = tempdir().unwrap();

        // Create existing manifest
        fs::write(dir.path().join("px.yaml"), "output: build").unwrap();

        let args = InitArgs {
            path: dir.path().to_path_buf(),
            force: false,
        };

        let result = run(args, &Printer::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_init_force_overwrites() {
        let dir = tempdir().unwrap();

        // Create existing manifest
        fs::write(dir.path().join("px.yaml"), "output: build").unwrap();

        let args = InitArgs {
            path: dir.path().to_path_buf(),
            force: true,
        };

        run(args, &Printer::new()).unwrap();

        let content = fs::read_to_string(dir.path().join("px.yaml")).unwrap();
        assert!(content.contains("output: dist"));
    }

    #[test]
    fn test_init_discovers_source_directories() {
        let dir = tempdir().unwrap();

        // Create nested structure
        fs::create_dir_all(dir.path().join("shapes")).unwrap();
        fs::create_dir_all(dir.path().join("palettes")).unwrap();

        fs::write(
            dir.path().join("shapes/wall.shape.md"),
            "---\nname: wall\n---\n\n```px\n#\n```",
        )
        .unwrap();
        fs::write(
            dir.path().join("palettes/game.palette.md"),
            "---\nname: game\n---\n$black: #000000",
        )
        .unwrap();

        let args = InitArgs {
            path: dir.path().to_path_buf(),
            force: false,
        };

        run(args, &Printer::new()).unwrap();

        let content = fs::read_to_string(dir.path().join("px.yaml")).unwrap();
        assert!(content.contains("sources:"));
        assert!(content.contains("palettes/"));
        assert!(content.contains("shapes/"));
    }

    #[test]
    fn test_init_empty_directory() {
        let dir = tempdir().unwrap();

        let args = InitArgs {
            path: dir.path().to_path_buf(),
            force: false,
        };

        run(args, &Printer::new()).unwrap();

        let content = fs::read_to_string(dir.path().join("px.yaml")).unwrap();
        assert!(content.contains("output: dist"));
        // No sources section needed for empty dir
        assert!(!content.contains("sources:"));
    }
}
