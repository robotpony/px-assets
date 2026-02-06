//! File system scanner for discovering px assets.
//!
//! Recursively scans directories to find all px definition files
//! (`.palette.md`, `.shape.md`, etc.).

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::registry::AssetKind;

use super::manifest::Manifest;

/// Result of scanning a directory for assets.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Discovered palette files.
    pub palettes: Vec<PathBuf>,
    /// Discovered stamp files.
    pub stamps: Vec<PathBuf>,
    /// Discovered brush files.
    pub brushes: Vec<PathBuf>,
    /// Discovered shader files.
    pub shaders: Vec<PathBuf>,
    /// Discovered shape files.
    pub shapes: Vec<PathBuf>,
    /// Discovered prefab files.
    pub prefabs: Vec<PathBuf>,
    /// Discovered map files.
    pub maps: Vec<PathBuf>,
}

impl ScanResult {
    /// Create a new empty scan result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the total number of discovered files.
    pub fn total(&self) -> usize {
        self.palettes.len()
            + self.stamps.len()
            + self.brushes.len()
            + self.shaders.len()
            + self.shapes.len()
            + self.prefabs.len()
            + self.maps.len()
    }

    /// Check if no files were discovered.
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }

    /// Get files of a specific asset kind.
    pub fn files_of_kind(&self, kind: AssetKind) -> &[PathBuf] {
        match kind {
            AssetKind::Palette => &self.palettes,
            AssetKind::Stamp => &self.stamps,
            AssetKind::Brush => &self.brushes,
            AssetKind::Shader => &self.shaders,
            AssetKind::Shape => &self.shapes,
            AssetKind::Prefab => &self.prefabs,
            AssetKind::Map => &self.maps,
        }
    }

    /// Merge another scan result into this one.
    pub fn merge(&mut self, other: ScanResult) {
        self.palettes.extend(other.palettes);
        self.stamps.extend(other.stamps);
        self.brushes.extend(other.brushes);
        self.shaders.extend(other.shaders);
        self.shapes.extend(other.shapes);
        self.prefabs.extend(other.prefabs);
        self.maps.extend(other.maps);
    }
}

/// Scan a directory for px asset files.
///
/// Recursively walks the directory and categorizes files by their
/// extension (e.g., `.shape.md`, `.palette.md`).
pub fn scan_directory(root: &Path, manifest: &Manifest) -> ScanResult {
    let mut result = ScanResult::new();

    if !root.exists() {
        return result;
    }

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Skip excluded paths
        if manifest.is_excluded(path) {
            continue;
        }

        // Check for px asset extensions
        if let Some(kind) = detect_asset_kind(path) {
            let path_buf = path.to_path_buf();
            match kind {
                AssetKind::Palette => result.palettes.push(path_buf),
                AssetKind::Stamp => result.stamps.push(path_buf),
                AssetKind::Brush => result.brushes.push(path_buf),
                AssetKind::Shader => result.shaders.push(path_buf),
                AssetKind::Shape => result.shapes.push(path_buf),
                AssetKind::Prefab => result.prefabs.push(path_buf),
                AssetKind::Map => result.maps.push(path_buf),
            }
        }
    }

    result
}

/// Scan multiple source paths.
pub fn scan_sources(sources: &[String], base_path: &Path, manifest: &Manifest) -> ScanResult {
    let mut result = ScanResult::new();

    for source in sources {
        let source_path = if Path::new(source).is_absolute() {
            PathBuf::from(source)
        } else {
            base_path.join(source)
        };

        let scan = scan_directory(&source_path, manifest);
        result.merge(scan);
    }

    result
}

/// Detect the asset kind from a file path based on its extension.
pub fn detect_asset_kind(path: &Path) -> Option<AssetKind> {
    let filename = path.file_name()?.to_str()?;

    // Check for double extensions like .shape.md
    if filename.ends_with(".palette.md") {
        Some(AssetKind::Palette)
    } else if filename.ends_with(".stamp.md") {
        Some(AssetKind::Stamp)
    } else if filename.ends_with(".brush.md") {
        Some(AssetKind::Brush)
    } else if filename.ends_with(".shader.md") {
        Some(AssetKind::Shader)
    } else if filename.ends_with(".shape.md") {
        Some(AssetKind::Shape)
    } else if filename.ends_with(".prefab.md") {
        Some(AssetKind::Prefab)
    } else if filename.ends_with(".map.md") {
        Some(AssetKind::Map)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_asset_kind() {
        assert_eq!(
            detect_asset_kind(Path::new("dungeon.palette.md")),
            Some(AssetKind::Palette)
        );
        assert_eq!(
            detect_asset_kind(Path::new("brick.stamp.md")),
            Some(AssetKind::Stamp)
        );
        assert_eq!(
            detect_asset_kind(Path::new("checker.brush.md")),
            Some(AssetKind::Brush)
        );
        assert_eq!(
            detect_asset_kind(Path::new("dark.shader.md")),
            Some(AssetKind::Shader)
        );
        assert_eq!(
            detect_asset_kind(Path::new("wall.shape.md")),
            Some(AssetKind::Shape)
        );
        assert_eq!(
            detect_asset_kind(Path::new("tower.prefab.md")),
            Some(AssetKind::Prefab)
        );
        assert_eq!(
            detect_asset_kind(Path::new("level-1.map.md")),
            Some(AssetKind::Map)
        );
        assert_eq!(detect_asset_kind(Path::new("readme.md")), None);
        assert_eq!(detect_asset_kind(Path::new("file.txt")), None);
    }

    #[test]
    fn test_detect_asset_kind_with_path() {
        assert_eq!(
            detect_asset_kind(Path::new("shapes/player/stand.shape.md")),
            Some(AssetKind::Shape)
        );
        assert_eq!(
            detect_asset_kind(Path::new("/absolute/path/game.palette.md")),
            Some(AssetKind::Palette)
        );
    }

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let manifest = Manifest::default();

        let result = scan_directory(dir.path(), &manifest);

        assert!(result.is_empty());
        assert_eq!(result.total(), 0);
    }

    #[test]
    fn test_scan_with_assets() {
        let dir = tempdir().unwrap();

        // Create test files
        fs::write(dir.path().join("game.palette.md"), "---\nname: game\n---").unwrap();
        fs::write(dir.path().join("player.shape.md"), "---\nname: player\n---").unwrap();
        fs::write(dir.path().join("wall.shape.md"), "---\nname: wall\n---").unwrap();
        fs::write(dir.path().join("readme.md"), "# Readme").unwrap();

        let manifest = Manifest::default();
        let result = scan_directory(dir.path(), &manifest);

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.shapes.len(), 2);
        assert_eq!(result.total(), 3);
    }

    #[test]
    fn test_scan_recursive() {
        let dir = tempdir().unwrap();

        // Create nested structure
        fs::create_dir_all(dir.path().join("shapes/player")).unwrap();
        fs::create_dir_all(dir.path().join("palettes")).unwrap();

        fs::write(
            dir.path().join("shapes/player/stand.shape.md"),
            "---\nname: stand\n---",
        )
        .unwrap();
        fs::write(
            dir.path().join("palettes/game.palette.md"),
            "---\nname: game\n---",
        )
        .unwrap();

        let manifest = Manifest::default();
        let result = scan_directory(dir.path(), &manifest);

        assert_eq!(result.shapes.len(), 1);
        assert_eq!(result.palettes.len(), 1);
    }

    #[test]
    fn test_scan_with_excludes() {
        let dir = tempdir().unwrap();

        fs::write(dir.path().join("player.shape.md"), "---\nname: player\n---").unwrap();
        fs::write(dir.path().join("backup.shape.md.bak"), "backup").unwrap();

        let manifest = Manifest {
            excludes: vec!["*.bak".to_string()],
            ..Default::default()
        };

        let result = scan_directory(dir.path(), &manifest);

        assert_eq!(result.shapes.len(), 1);
        assert!(result.shapes[0].to_string_lossy().contains("player"));
    }

    #[test]
    fn test_scan_result_merge() {
        let mut a = ScanResult::new();
        a.shapes.push(PathBuf::from("a.shape.md"));

        let mut b = ScanResult::new();
        b.shapes.push(PathBuf::from("b.shape.md"));
        b.palettes.push(PathBuf::from("c.palette.md"));

        a.merge(b);

        assert_eq!(a.shapes.len(), 2);
        assert_eq!(a.palettes.len(), 1);
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let manifest = Manifest::default();
        let result = scan_directory(Path::new("/nonexistent/path"), &manifest);

        assert!(result.is_empty());
    }

    #[test]
    fn test_files_of_kind() {
        let mut result = ScanResult::new();
        result.shapes.push(PathBuf::from("a.shape.md"));
        result.palettes.push(PathBuf::from("b.palette.md"));

        assert_eq!(result.files_of_kind(AssetKind::Shape).len(), 1);
        assert_eq!(result.files_of_kind(AssetKind::Palette).len(), 1);
        assert_eq!(result.files_of_kind(AssetKind::Brush).len(), 0);
    }
}
