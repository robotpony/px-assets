//! Project manifest (px.yaml) parsing.
//!
//! The manifest defines project configuration including source paths,
//! output settings, and default options.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{PxError, Result};

/// Project manifest loaded from px.yaml.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
    /// Source directories or glob patterns to scan for assets.
    /// Defaults to current directory if empty.
    #[serde(default)]
    pub sources: Vec<String>,

    /// Output directory for rendered assets.
    #[serde(default = "default_output")]
    pub output: PathBuf,

    /// Default target name (e.g., "web", "pico8").
    #[serde(default)]
    pub target: Option<String>,

    /// Default shader name.
    #[serde(default)]
    pub shader: Option<String>,

    /// Default scale factor for output.
    #[serde(default)]
    pub scale: Option<u32>,

    /// Patterns to exclude from discovery.
    #[serde(default)]
    pub excludes: Vec<String>,
}

fn default_output() -> PathBuf {
    PathBuf::from("dist")
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            sources: vec![],
            output: default_output(),
            target: None,
            shader: None,
            scale: None,
            excludes: vec![],
        }
    }
}

impl Manifest {
    /// Load manifest from a px.yaml file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| PxError::Io {
            path: path.to_path_buf(),
            message: format!("Failed to read manifest: {}", e),
        })?;

        Self::parse(&content)
    }

    /// Parse manifest from YAML string.
    pub fn parse(content: &str) -> Result<Self> {
        serde_yaml::from_str(content).map_err(|e| PxError::Parse {
            message: format!("Invalid manifest: {}", e),
            help: Some("Check px.yaml syntax".to_string()),
        })
    }

    /// Check if a path should be excluded based on exclude patterns.
    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.excludes {
            // Simple glob matching: * matches any sequence
            if Self::matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Simple glob pattern matching.
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        // Handle common patterns
        if pattern.starts_with("**/") {
            // Match anywhere in path: **/foo/* matches any path containing /foo/
            let suffix = &pattern[3..];
            if suffix.ends_with("/*") {
                // **/dir/* matches anything inside dir anywhere in the path
                let dir = &suffix[..suffix.len() - 2];
                return path.contains(&format!("{}/", dir))
                    || path.contains(&format!("/{}/", dir))
                    || path.starts_with(&format!("{}/", dir));
            }
            return path.contains(suffix) || path.ends_with(suffix);
        }

        if pattern.starts_with('*') && !pattern.contains('/') {
            // Match file extension or suffix
            let suffix = &pattern[1..];
            return path.ends_with(suffix);
        }

        if pattern.ends_with("/*") {
            // Match directory contents
            let prefix = &pattern[..pattern.len() - 2];
            return path.starts_with(&format!("{}/", prefix))
                || path.contains(&format!("/{}/", prefix));
        }

        // Exact match or contains
        path.contains(pattern)
    }

    /// Get effective source paths, defaulting to current directory.
    pub fn effective_sources(&self) -> Vec<String> {
        if self.sources.is_empty() {
            vec![".".to_string()]
        } else {
            self.sources.clone()
        }
    }

    /// Get the effective scale factor.
    pub fn effective_scale(&self) -> u32 {
        self.scale.unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let yaml = "output: build";
        let manifest = Manifest::parse(yaml).unwrap();

        assert_eq!(manifest.output, PathBuf::from("build"));
        assert!(manifest.sources.is_empty());
        assert!(manifest.target.is_none());
    }

    #[test]
    fn test_parse_full_manifest() {
        let yaml = r#"
sources:
  - shapes/
  - palettes/
output: dist/sprites
target: web
shader: dungeon-dark
scale: 4
excludes:
  - "*.bak"
  - "**/temp/*"
"#;
        let manifest = Manifest::parse(yaml).unwrap();

        assert_eq!(manifest.sources, vec!["shapes/", "palettes/"]);
        assert_eq!(manifest.output, PathBuf::from("dist/sprites"));
        assert_eq!(manifest.target, Some("web".to_string()));
        assert_eq!(manifest.shader, Some("dungeon-dark".to_string()));
        assert_eq!(manifest.scale, Some(4));
        assert_eq!(manifest.excludes, vec!["*.bak", "**/temp/*"]);
    }

    #[test]
    fn test_default_manifest() {
        let manifest = Manifest::default();

        assert!(manifest.sources.is_empty());
        assert_eq!(manifest.output, PathBuf::from("dist"));
        assert!(manifest.target.is_none());
        assert!(manifest.shader.is_none());
        assert!(manifest.scale.is_none());
        assert!(manifest.excludes.is_empty());
    }

    #[test]
    fn test_effective_sources() {
        let mut manifest = Manifest::default();
        assert_eq!(manifest.effective_sources(), vec!["."]);

        manifest.sources = vec!["src/".to_string()];
        assert_eq!(manifest.effective_sources(), vec!["src/"]);
    }

    #[test]
    fn test_is_excluded_extension() {
        let manifest = Manifest {
            excludes: vec!["*.bak".to_string()],
            ..Default::default()
        };

        assert!(manifest.is_excluded(Path::new("file.bak")));
        assert!(manifest.is_excluded(Path::new("path/to/file.bak")));
        assert!(!manifest.is_excluded(Path::new("file.md")));
    }

    #[test]
    fn test_is_excluded_directory() {
        let manifest = Manifest {
            excludes: vec!["**/node_modules/*".to_string()],
            ..Default::default()
        };

        assert!(manifest.is_excluded(Path::new("node_modules/foo")));
        assert!(manifest.is_excluded(Path::new("path/node_modules/bar")));
        assert!(!manifest.is_excluded(Path::new("src/file.md")));
    }

    #[test]
    fn test_is_excluded_exact() {
        let manifest = Manifest {
            excludes: vec!["temp".to_string()],
            ..Default::default()
        };

        assert!(manifest.is_excluded(Path::new("temp")));
        assert!(manifest.is_excluded(Path::new("path/temp/file")));
    }

    #[test]
    fn test_parse_empty_manifest() {
        let yaml = "";
        let manifest = Manifest::parse(yaml).unwrap();

        // Should use defaults
        assert_eq!(manifest.output, PathBuf::from("dist"));
    }
}
