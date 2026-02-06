//! File discovery and asset loading for px projects.
//!
//! This module handles finding and loading all px definition files
//! from a project directory, either using convention-based discovery
//! or a `px.yaml` manifest.
//!
//! # Example
//!
//! ```ignore
//! use px::discovery::discover;
//!
//! let result = discover("./my-project")?;
//! println!("Found {} assets", result.scan.total());
//!
//! let registry = result.into_registry()?;
//! ```

mod loader;
mod manifest;
mod scanner;

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::registry::{AssetRegistry, RegistryBuilder};

pub use loader::{load_assets, LoadOptions};
pub use manifest::Manifest;
pub use scanner::{detect_asset_kind, scan_directory, scan_sources, ScanResult};

/// The name of the manifest file.
pub const MANIFEST_FILENAME: &str = "px.yaml";

/// Result of discovering assets in a project.
#[derive(Debug)]
pub struct DiscoveryResult {
    /// The project root directory.
    pub root: PathBuf,

    /// The loaded manifest (may be default if no px.yaml found).
    pub manifest: Manifest,

    /// Whether a px.yaml manifest was found.
    pub has_manifest: bool,

    /// Scan results with discovered files.
    pub scan: ScanResult,
}

impl DiscoveryResult {
    /// Load all discovered assets and build a registry.
    pub fn into_registry(self) -> Result<AssetRegistry> {
        self.into_registry_with_options(LoadOptions::with_builtins())
    }

    /// Load all discovered assets with custom options.
    pub fn into_registry_with_options(self, options: LoadOptions) -> Result<AssetRegistry> {
        let builder = load_assets(&self.scan, &options)?;
        builder.build()
    }

    /// Get a registry builder with loaded assets (for custom configuration).
    pub fn into_builder(self) -> Result<RegistryBuilder> {
        self.into_builder_with_options(LoadOptions::with_builtins())
    }

    /// Get a registry builder with custom options.
    pub fn into_builder_with_options(self, options: LoadOptions) -> Result<RegistryBuilder> {
        load_assets(&self.scan, &options)
    }
}

/// Discover assets in a project directory.
///
/// Looks for a `px.yaml` manifest in the root directory. If found, uses
/// the manifest's source paths. Otherwise, scans the entire directory
/// for px asset files.
///
/// Returns a `DiscoveryResult` containing the manifest, scan results,
/// and methods to load assets into a registry.
pub fn discover(root: impl AsRef<Path>) -> Result<DiscoveryResult> {
    let root = root.as_ref().to_path_buf();

    // Look for manifest
    let manifest_path = root.join(MANIFEST_FILENAME);
    let (manifest, has_manifest) = if manifest_path.exists() {
        (Manifest::load(&manifest_path)?, true)
    } else {
        (Manifest::default(), false)
    };

    // Scan for assets
    let sources = manifest.effective_sources();
    let scan = scan_sources(&sources, &root, &manifest);

    Ok(DiscoveryResult {
        root,
        manifest,
        has_manifest,
        scan,
    })
}

/// Discover assets from specific paths (no manifest lookup).
///
/// Useful when you want to specify source paths directly without
/// looking for a px.yaml manifest.
pub fn discover_paths(paths: &[PathBuf]) -> Result<DiscoveryResult> {
    let manifest = Manifest::default();
    let mut scan = ScanResult::new();

    for path in paths {
        if path.is_dir() {
            let dir_scan = scan_directory(path, &manifest);
            scan.merge(dir_scan);
        } else if path.is_file() {
            // Add single file to appropriate category
            if let Some(kind) = detect_asset_kind(path) {
                match kind {
                    crate::registry::AssetKind::Palette => scan.palettes.push(path.clone()),
                    crate::registry::AssetKind::Stamp => scan.stamps.push(path.clone()),
                    crate::registry::AssetKind::Brush => scan.brushes.push(path.clone()),
                    crate::registry::AssetKind::Shader => scan.shaders.push(path.clone()),
                    crate::registry::AssetKind::Shape => scan.shapes.push(path.clone()),
                    crate::registry::AssetKind::Prefab => scan.prefabs.push(path.clone()),
                    crate::registry::AssetKind::Map => scan.maps.push(path.clone()),
                }
            }
        }
    }

    let root = paths
        .first()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    Ok(DiscoveryResult {
        root,
        manifest,
        has_manifest: false,
        scan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_discover_empty_directory() {
        let dir = tempdir().unwrap();

        let result = discover(dir.path()).unwrap();

        assert!(!result.has_manifest);
        assert!(result.scan.is_empty());
    }

    #[test]
    fn test_discover_without_manifest() {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("player.shape.md"),
            "---\nname: player\n---\n\n```px\n#\n```",
        )
        .unwrap();

        let result = discover(dir.path()).unwrap();

        assert!(!result.has_manifest);
        assert_eq!(result.scan.shapes.len(), 1);
    }

    #[test]
    fn test_discover_with_manifest() {
        let dir = tempdir().unwrap();

        // Create manifest
        fs::write(
            dir.path().join("px.yaml"),
            r#"
sources:
  - assets/
output: build
scale: 2
"#,
        )
        .unwrap();

        // Create assets directory with files
        fs::create_dir_all(dir.path().join("assets")).unwrap();
        fs::write(
            dir.path().join("assets/game.palette.md"),
            "---\nname: game\n---\n$black: #000000",
        )
        .unwrap();

        let result = discover(dir.path()).unwrap();

        assert!(result.has_manifest);
        assert_eq!(result.manifest.scale, Some(2));
        assert_eq!(result.manifest.output, PathBuf::from("build"));
        assert_eq!(result.scan.palettes.len(), 1);
    }

    #[test]
    fn test_discover_with_excludes() {
        let dir = tempdir().unwrap();

        // Create manifest with excludes
        fs::write(
            dir.path().join("px.yaml"),
            r#"
excludes:
  - "**/backup/*"
"#,
        )
        .unwrap();

        // Create files
        fs::write(
            dir.path().join("player.shape.md"),
            "---\nname: player\n---\n\n```px\n#\n```",
        )
        .unwrap();

        fs::create_dir_all(dir.path().join("backup")).unwrap();
        fs::write(
            dir.path().join("backup/old.shape.md"),
            "---\nname: old\n---\n\n```px\n#\n```",
        )
        .unwrap();

        let result = discover(dir.path()).unwrap();

        // Should only find player.shape.md, not backup/old.shape.md
        assert_eq!(result.scan.shapes.len(), 1);
        assert!(result.scan.shapes[0].to_string_lossy().contains("player"));
    }

    #[test]
    fn test_discover_into_registry() {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("test.shape.md"),
            "---\nname: test\n---\n\n```px\n##\n```",
        )
        .unwrap();

        let result = discover(dir.path()).unwrap();
        let registry = result.into_registry().unwrap();

        assert!(registry.get_shape("test").is_some());
        // Should also have builtins
        assert!(registry.get_stamp("corner").is_some());
    }

    #[test]
    fn test_discover_paths_files() {
        let dir = tempdir().unwrap();

        let shape_path = dir.path().join("test.shape.md");
        fs::write(&shape_path, "---\nname: test\n---\n\n```px\n#\n```").unwrap();

        let result = discover_paths(&[shape_path]).unwrap();

        assert_eq!(result.scan.shapes.len(), 1);
    }

    #[test]
    fn test_discover_paths_directories() {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("test.shape.md"),
            "---\nname: test\n---\n\n```px\n#\n```",
        )
        .unwrap();

        let result = discover_paths(&[dir.path().to_path_buf()]).unwrap();

        assert_eq!(result.scan.shapes.len(), 1);
    }

    #[test]
    fn test_discover_into_builder() {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("test.shape.md"),
            "---\nname: test\n---\n\n```px\n#\n```",
        )
        .unwrap();

        let result = discover(dir.path()).unwrap();
        let builder = result.into_builder().unwrap();

        // Can customize before building
        let registry = builder.build().unwrap();
        assert!(registry.get_shape("test").is_some());
    }

    #[test]
    fn test_manifest_effective_scale() {
        let manifest = Manifest {
            scale: Some(4),
            ..Default::default()
        };
        assert_eq!(manifest.effective_scale(), 4);

        let manifest = Manifest::default();
        assert_eq!(manifest.effective_scale(), 1);
    }
}
