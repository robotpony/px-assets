//! Colour type and parsing.

use std::fmt;
use std::str::FromStr;

use crate::error::{PxError, Result};

/// An RGBA colour value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    /// Create a new colour from RGBA components.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new opaque colour from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Fully transparent colour.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    /// Black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    /// White.
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    /// Magenta (used for missing/error placeholders).
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);

    /// Parse a hex colour string.
    ///
    /// Supports formats:
    /// - `#RGB` (3 digits, expanded to 6)
    /// - `#RGBA` (4 digits, expanded to 8)
    /// - `#RRGGBB` (6 digits)
    /// - `#RRGGBBAA` (8 digits)
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.trim();
        let hex = s.strip_prefix('#').unwrap_or(s);

        match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = parse_hex_digit(hex.chars().nth(0).unwrap())?;
                let g = parse_hex_digit(hex.chars().nth(1).unwrap())?;
                let b = parse_hex_digit(hex.chars().nth(2).unwrap())?;
                Ok(Self::rgb(r << 4 | r, g << 4 | g, b << 4 | b))
            }
            4 => {
                // #RGBA -> #RRGGBBAA
                let r = parse_hex_digit(hex.chars().nth(0).unwrap())?;
                let g = parse_hex_digit(hex.chars().nth(1).unwrap())?;
                let b = parse_hex_digit(hex.chars().nth(2).unwrap())?;
                let a = parse_hex_digit(hex.chars().nth(3).unwrap())?;
                Ok(Self::new(r << 4 | r, g << 4 | g, b << 4 | b, a << 4 | a))
            }
            6 => {
                // #RRGGBB
                let r = parse_hex_byte(&hex[0..2])?;
                let g = parse_hex_byte(&hex[2..4])?;
                let b = parse_hex_byte(&hex[4..6])?;
                Ok(Self::rgb(r, g, b))
            }
            8 => {
                // #RRGGBBAA
                let r = parse_hex_byte(&hex[0..2])?;
                let g = parse_hex_byte(&hex[2..4])?;
                let b = parse_hex_byte(&hex[4..6])?;
                let a = parse_hex_byte(&hex[6..8])?;
                Ok(Self::new(r, g, b, a))
            }
            _ => Err(PxError::Parse {
                message: format!("Invalid hex colour: {}", s),
                help: Some("Use #RGB, #RGBA, #RRGGBB, or #RRGGBBAA format".to_string()),
            }),
        }
    }

    /// Convert to RGBA tuple.
    pub fn to_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Check if the colour is fully transparent.
    pub fn is_transparent(self) -> bool {
        self.a == 0
    }

    /// Check if the colour is fully opaque.
    pub fn is_opaque(self) -> bool {
        self.a == 255
    }
}

impl FromStr for Colour {
    type Err = PxError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_hex(s)
    }
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            write!(f, "#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

/// Parse a single hex digit.
fn parse_hex_digit(c: char) -> Result<u8> {
    c.to_digit(16)
        .map(|d| d as u8)
        .ok_or_else(|| PxError::Parse {
            message: format!("Invalid hex digit: {}", c),
            help: None,
        })
}

/// Parse a two-character hex byte.
fn parse_hex_byte(s: &str) -> Result<u8> {
    u8::from_str_radix(s, 16).map_err(|_| PxError::Parse {
        message: format!("Invalid hex byte: {}", s),
        help: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_6digit() {
        let c = Colour::from_hex("#FF0000").unwrap();
        assert_eq!(c, Colour::rgb(255, 0, 0));

        let c = Colour::from_hex("#1a1a2e").unwrap();
        assert_eq!(c, Colour::rgb(0x1a, 0x1a, 0x2e));
    }

    #[test]
    fn test_from_hex_3digit() {
        let c = Colour::from_hex("#F00").unwrap();
        assert_eq!(c, Colour::rgb(255, 0, 0));

        let c = Colour::from_hex("#ABC").unwrap();
        assert_eq!(c, Colour::rgb(0xAA, 0xBB, 0xCC));
    }

    #[test]
    fn test_from_hex_8digit() {
        let c = Colour::from_hex("#FF000080").unwrap();
        assert_eq!(c, Colour::new(255, 0, 0, 128));
    }

    #[test]
    fn test_from_hex_4digit() {
        let c = Colour::from_hex("#F008").unwrap();
        assert_eq!(c, Colour::new(255, 0, 0, 136)); // 0x88
    }

    #[test]
    fn test_from_hex_no_hash() {
        let c = Colour::from_hex("FF0000").unwrap();
        assert_eq!(c, Colour::rgb(255, 0, 0));
    }

    #[test]
    fn test_from_hex_invalid() {
        assert!(Colour::from_hex("#GGG").is_err());
        assert!(Colour::from_hex("#12345").is_err());
        assert!(Colour::from_hex("").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Colour::rgb(255, 0, 0)), "#FF0000");
        assert_eq!(format!("{}", Colour::new(255, 0, 0, 128)), "#FF000080");
    }

    #[test]
    fn test_constants() {
        assert_eq!(Colour::BLACK, Colour::rgb(0, 0, 0));
        assert_eq!(Colour::WHITE, Colour::rgb(255, 255, 255));
        assert!(Colour::TRANSPARENT.is_transparent());
        assert!(Colour::BLACK.is_opaque());
    }
}
