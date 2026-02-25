//! Target type for output configuration profiles.
//!
//! Targets bundle output settings (format, scale, sheet mode, padding, shader)
//! into named profiles. CLI flags override target settings.
//!
//! # Example
//!
//! ```yaml
//! ---
//! name: web
//! format: png
//! ---
//! scale: 2
//! sheet: auto
//! padding: 1
//! shader: default
//! ```

/// How sprites are packed into sheets.
#[derive(Debug, Clone, PartialEq)]
pub enum SheetConfig {
    /// No sheet packing; output individual files.
    None,
    /// Automatically determine sheet layout.
    Auto,
    /// Fixed sheet dimensions (in tiles).
    Fixed { width: u32, height: u32 },
}

impl SheetConfig {
    /// Parse a sheet config from a string.
    ///
    /// Accepts "none", "auto", or "WxH" (e.g. "8x4").
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "none" | "false" => Ok(SheetConfig::None),
            "auto" | "true" => Ok(SheetConfig::Auto),
            _ => {
                // Try WxH format
                let parts: Vec<&str> = s.split('x').collect();
                if parts.len() == 2 {
                    let w = parts[0]
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid sheet width: '{}'", parts[0]))?;
                    let h = parts[1]
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| format!("Invalid sheet height: '{}'", parts[1]))?;
                    Ok(SheetConfig::Fixed { width: w, height: h })
                } else {
                    Err(format!(
                        "Invalid sheet config: '{}' (expected 'none', 'auto', or 'WxH')",
                        s
                    ))
                }
            }
        }
    }
}

/// How palette colours are stored in output.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteMode {
    /// Full RGBA colour values.
    Rgba,
    /// Indexed palette (for formats that support it).
    Indexed,
}

/// An output configuration profile.
#[derive(Debug, Clone)]
pub struct Target {
    /// Target name (unique identifier).
    pub name: String,
    /// Output format (currently only "png").
    pub format: String,
    /// Scale factor for output.
    pub scale: Option<u32>,
    /// Sheet packing mode.
    pub sheet: SheetConfig,
    /// Padding between sprites in sheet (pixels).
    pub padding: Option<u32>,
    /// Palette output mode.
    pub palette_mode: PaletteMode,
    /// Shader to use for rendering.
    pub shader: Option<String>,
}

impl Target {
    /// Create a new target with minimal settings.
    pub fn new(name: impl Into<String>, format: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            format: format.into(),
            scale: None,
            sheet: SheetConfig::None,
            padding: None,
            palette_mode: PaletteMode::Rgba,
            shader: None,
        }
    }
}

/// Collection of builtin targets.
pub struct BuiltinTargets;

impl BuiltinTargets {
    /// Get the "web" target: PNG, no sheet, all defaults.
    fn web() -> Target {
        Target::new("web", "png")
    }

    /// Get the "sheet" target: PNG, auto sheet packing.
    fn sheet() -> Target {
        Target {
            name: "sheet".to_string(),
            format: "png".to_string(),
            scale: None,
            sheet: SheetConfig::Auto,
            padding: None,
            palette_mode: PaletteMode::Rgba,
            shader: None,
        }
    }

    /// Get the "p8" target: PICO-8 cartridge, fixed 128x128 sheet.
    fn p8() -> Target {
        Target {
            name: "p8".to_string(),
            format: "p8".to_string(),
            scale: Some(1),
            sheet: SheetConfig::Fixed { width: 128, height: 128 },
            padding: Some(0),
            palette_mode: PaletteMode::Indexed,
            shader: None,
        }
    }

    /// Get a builtin target by name.
    pub fn get(name: &str) -> Option<Target> {
        match name {
            "web" => Some(Self::web()),
            "sheet" => Some(Self::sheet()),
            "p8" => Some(Self::p8()),
            _ => None,
        }
    }

    /// Get all builtin targets.
    pub fn all() -> Vec<Target> {
        vec![Self::web(), Self::sheet(), Self::p8()]
    }
}

/// Builder for constructing targets from parsed definitions.
#[derive(Debug, Clone)]
pub struct TargetBuilder {
    name: String,
    format: Option<String>,
    scale: Option<u32>,
    sheet: Option<SheetConfig>,
    padding: Option<u32>,
    palette_mode: Option<PaletteMode>,
    shader: Option<String>,
}

impl TargetBuilder {
    /// Create a new target builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            format: None,
            scale: None,
            sheet: None,
            padding: None,
            palette_mode: None,
            shader: None,
        }
    }

    /// Set the output format.
    pub fn format(&mut self, format: impl Into<String>) -> &mut Self {
        self.format = Some(format.into());
        self
    }

    /// Set the scale factor.
    pub fn scale(&mut self, scale: u32) -> &mut Self {
        self.scale = Some(scale);
        self
    }

    /// Set the sheet config.
    pub fn sheet(&mut self, sheet: SheetConfig) -> &mut Self {
        self.sheet = Some(sheet);
        self
    }

    /// Set the padding.
    pub fn padding(&mut self, padding: u32) -> &mut Self {
        self.padding = Some(padding);
        self
    }

    /// Set the palette mode.
    pub fn palette_mode(&mut self, mode: PaletteMode) -> &mut Self {
        self.palette_mode = Some(mode);
        self
    }

    /// Set the shader.
    pub fn shader(&mut self, shader: impl Into<String>) -> &mut Self {
        self.shader = Some(shader.into());
        self
    }

    /// Build the target.
    pub fn build(self) -> Result<Target, &'static str> {
        let format = self.format.unwrap_or_else(|| "png".to_string());

        Ok(Target {
            name: self.name,
            format,
            scale: self.scale,
            sheet: self.sheet.unwrap_or(SheetConfig::None),
            padding: self.padding,
            palette_mode: self.palette_mode.unwrap_or(PaletteMode::Rgba),
            shader: self.shader,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheet_config_parse_none() {
        assert_eq!(SheetConfig::parse("none").unwrap(), SheetConfig::None);
        assert_eq!(SheetConfig::parse("false").unwrap(), SheetConfig::None);
    }

    #[test]
    fn test_sheet_config_parse_auto() {
        assert_eq!(SheetConfig::parse("auto").unwrap(), SheetConfig::Auto);
        assert_eq!(SheetConfig::parse("true").unwrap(), SheetConfig::Auto);
    }

    #[test]
    fn test_sheet_config_parse_fixed() {
        assert_eq!(
            SheetConfig::parse("8x4").unwrap(),
            SheetConfig::Fixed {
                width: 8,
                height: 4
            }
        );
    }

    #[test]
    fn test_sheet_config_parse_invalid() {
        assert!(SheetConfig::parse("banana").is_err());
        assert!(SheetConfig::parse("8xfoo").is_err());
    }

    #[test]
    fn test_target_new() {
        let target = Target::new("test", "png");
        assert_eq!(target.name, "test");
        assert_eq!(target.format, "png");
        assert_eq!(target.scale, None);
        assert_eq!(target.sheet, SheetConfig::None);
        assert_eq!(target.padding, None);
        assert_eq!(target.palette_mode, PaletteMode::Rgba);
        assert_eq!(target.shader, None);
    }

    #[test]
    fn test_builtin_web() {
        let target = BuiltinTargets::get("web").unwrap();
        assert_eq!(target.name, "web");
        assert_eq!(target.format, "png");
        assert_eq!(target.sheet, SheetConfig::None);
    }

    #[test]
    fn test_builtin_sheet() {
        let target = BuiltinTargets::get("sheet").unwrap();
        assert_eq!(target.name, "sheet");
        assert_eq!(target.sheet, SheetConfig::Auto);
    }

    #[test]
    fn test_builtin_p8() {
        let target = BuiltinTargets::get("p8").unwrap();
        assert_eq!(target.name, "p8");
        assert_eq!(target.format, "p8");
        assert_eq!(target.scale, Some(1));
        assert_eq!(target.sheet, SheetConfig::Fixed { width: 128, height: 128 });
        assert_eq!(target.padding, Some(0));
        assert_eq!(target.palette_mode, PaletteMode::Indexed);
    }

    #[test]
    fn test_builtin_unknown() {
        assert!(BuiltinTargets::get("aseprite").is_none());
    }

    #[test]
    fn test_target_builder() {
        let mut builder = TargetBuilder::new("custom");
        builder.format("png");
        builder.scale(2);
        builder.sheet(SheetConfig::Auto);
        builder.padding(1);
        builder.shader("dark");

        let target = builder.build().unwrap();
        assert_eq!(target.name, "custom");
        assert_eq!(target.format, "png");
        assert_eq!(target.scale, Some(2));
        assert_eq!(target.sheet, SheetConfig::Auto);
        assert_eq!(target.padding, Some(1));
        assert_eq!(target.shader, Some("dark".to_string()));
    }

    #[test]
    fn test_target_builder_defaults() {
        let builder = TargetBuilder::new("minimal");
        let target = builder.build().unwrap();

        assert_eq!(target.format, "png");
        assert_eq!(target.sheet, SheetConfig::None);
        assert_eq!(target.palette_mode, PaletteMode::Rgba);
    }
}
