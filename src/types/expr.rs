//! Colour expression parsing and evaluation.
//!
//! Supports function-style colour expressions:
//! - `darken($gold, 20%)` - reduce lightness by percentage
//! - `lighten($gold, 20%)` - increase lightness by percentage
//! - `saturate($gold, 20%)` - increase saturation by percentage
//! - `desaturate($gold, 20%)` - decrease saturation by percentage
//! - `mix($a, $b, 50%)` - blend two colours
//! - `alpha($gold, 50%)` - set alpha channel

use crate::error::{PxError, Result};
use crate::types::Colour;

/// A parsed colour expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ColourExpr {
    /// A hex literal: `#FF0000`
    Hex(String),
    /// A reference to another colour: `$gold`
    Reference(String),
    /// A function call: `darken($gold, 20%)`
    Function {
        name: String,
        args: Vec<ColourExpr>,
    },
    /// A percentage value (used as argument): `20%`
    Percent(f32),
}

impl ColourExpr {
    /// Parse a colour expression from a string.
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        if input.is_empty() {
            return Err(PxError::Parse {
                message: "Empty colour expression".to_string(),
                help: None,
            });
        }

        // Hex literal
        if input.starts_with('#') {
            return Ok(ColourExpr::Hex(input.to_string()));
        }

        // Percentage
        if input.ends_with('%') {
            let num_str = input.trim_end_matches('%');
            let value: f32 = num_str.parse().map_err(|_| PxError::Parse {
                message: format!("Invalid percentage: {}", input),
                help: Some("Use format like 20% or 50.5%".to_string()),
            })?;
            return Ok(ColourExpr::Percent(value));
        }

        // Function call: name(args)
        if let Some(paren_pos) = input.find('(') {
            if !input.ends_with(')') {
                return Err(PxError::Parse {
                    message: format!("Unclosed function call: {}", input),
                    help: Some("Add closing parenthesis".to_string()),
                });
            }

            let name = input[..paren_pos].trim().to_string();
            let args_str = &input[paren_pos + 1..input.len() - 1];

            // Parse arguments (comma-separated)
            let args = parse_args(args_str)?;

            return Ok(ColourExpr::Function { name, args });
        }

        // Reference: $name or just name
        if input.starts_with('$') {
            Ok(ColourExpr::Reference(input[1..].to_string()))
        } else {
            // Could be a bare name reference (less common)
            Ok(ColourExpr::Reference(input.to_string()))
        }
    }

    /// Check if this is a simple value (hex or reference, not a function).
    pub fn is_simple(&self) -> bool {
        matches!(self, ColourExpr::Hex(_) | ColourExpr::Reference(_))
    }
}

/// Parse comma-separated arguments, handling nested parentheses.
fn parse_args(input: &str) -> Result<Vec<ColourExpr>> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(vec![]);
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;

    for c in input.chars() {
        match c {
            '(' => {
                paren_depth += 1;
                current.push(c);
            }
            ')' => {
                paren_depth -= 1;
                current.push(c);
            }
            ',' if paren_depth == 0 => {
                let arg = current.trim().to_string();
                if !arg.is_empty() {
                    args.push(ColourExpr::parse(&arg)?);
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Don't forget the last argument
    let arg = current.trim().to_string();
    if !arg.is_empty() {
        args.push(ColourExpr::parse(&arg)?);
    }

    Ok(args)
}

/// Colour expression evaluator.
///
/// Evaluates expressions by resolving references through a lookup function.
pub struct ExprEvaluator<F>
where
    F: Fn(&str) -> Option<Colour>,
{
    lookup: F,
}

impl<F> ExprEvaluator<F>
where
    F: Fn(&str) -> Option<Colour>,
{
    /// Create a new evaluator with the given colour lookup function.
    pub fn new(lookup: F) -> Self {
        Self { lookup }
    }

    /// Evaluate an expression to a colour.
    pub fn eval(&self, expr: &ColourExpr) -> Result<Colour> {
        match expr {
            ColourExpr::Hex(hex) => Colour::from_hex(hex),

            ColourExpr::Reference(name) => (self.lookup)(name).ok_or_else(|| PxError::Parse {
                message: format!("Undefined colour: ${}", name),
                help: None,
            }),

            ColourExpr::Percent(_) => Err(PxError::Parse {
                message: "Percentage cannot be evaluated as a colour".to_string(),
                help: Some("Percentages are only valid as function arguments".to_string()),
            }),

            ColourExpr::Function { name, args } => self.eval_function(name, args),
        }
    }

    /// Evaluate a function call.
    fn eval_function(&self, name: &str, args: &[ColourExpr]) -> Result<Colour> {
        match name {
            "darken" => self.eval_darken(args),
            "lighten" => self.eval_lighten(args),
            "saturate" => self.eval_saturate(args),
            "desaturate" => self.eval_desaturate(args),
            "mix" => self.eval_mix(args),
            "alpha" => self.eval_alpha(args),
            _ => Err(PxError::Parse {
                message: format!("Unknown colour function: {}", name),
                help: Some(
                    "Available functions: darken, lighten, saturate, desaturate, mix, alpha"
                        .to_string(),
                ),
            }),
        }
    }

    /// darken($colour, percent) - reduce lightness
    fn eval_darken(&self, args: &[ColourExpr]) -> Result<Colour> {
        let (colour, percent) = self.expect_colour_and_percent(args, "darken")?;
        Ok(adjust_lightness(colour, -percent))
    }

    /// lighten($colour, percent) - increase lightness
    fn eval_lighten(&self, args: &[ColourExpr]) -> Result<Colour> {
        let (colour, percent) = self.expect_colour_and_percent(args, "lighten")?;
        Ok(adjust_lightness(colour, percent))
    }

    /// saturate($colour, percent) - increase saturation
    fn eval_saturate(&self, args: &[ColourExpr]) -> Result<Colour> {
        let (colour, percent) = self.expect_colour_and_percent(args, "saturate")?;
        Ok(adjust_saturation(colour, percent))
    }

    /// desaturate($colour, percent) - decrease saturation
    fn eval_desaturate(&self, args: &[ColourExpr]) -> Result<Colour> {
        let (colour, percent) = self.expect_colour_and_percent(args, "desaturate")?;
        Ok(adjust_saturation(colour, -percent))
    }

    /// mix($colour1, $colour2, percent) - blend colours
    fn eval_mix(&self, args: &[ColourExpr]) -> Result<Colour> {
        if args.len() != 3 {
            return Err(PxError::Parse {
                message: format!("mix() requires 3 arguments, got {}", args.len()),
                help: Some("Usage: mix($colour1, $colour2, 50%)".to_string()),
            });
        }

        let colour1 = self.eval(&args[0])?;
        let colour2 = self.eval(&args[1])?;
        let percent = self.expect_percent(&args[2], "mix")?;

        Ok(mix_colours(colour1, colour2, percent / 100.0))
    }

    /// alpha($colour, percent) - set alpha channel
    fn eval_alpha(&self, args: &[ColourExpr]) -> Result<Colour> {
        let (colour, percent) = self.expect_colour_and_percent(args, "alpha")?;
        let alpha = ((percent / 100.0) * 255.0).clamp(0.0, 255.0) as u8;
        Ok(Colour::new(colour.r, colour.g, colour.b, alpha))
    }

    /// Helper: expect (colour, percent) arguments
    fn expect_colour_and_percent(
        &self,
        args: &[ColourExpr],
        func_name: &str,
    ) -> Result<(Colour, f32)> {
        if args.len() != 2 {
            return Err(PxError::Parse {
                message: format!("{}() requires 2 arguments, got {}", func_name, args.len()),
                help: Some(format!("Usage: {}($colour, 20%)", func_name)),
            });
        }

        let colour = self.eval(&args[0])?;
        let percent = self.expect_percent(&args[1], func_name)?;

        Ok((colour, percent))
    }

    /// Helper: expect a percentage argument
    fn expect_percent(&self, expr: &ColourExpr, func_name: &str) -> Result<f32> {
        match expr {
            ColourExpr::Percent(p) => Ok(*p),
            _ => Err(PxError::Parse {
                message: format!("{}() requires a percentage argument", func_name),
                help: Some(format!("Usage: {}($colour, 20%)", func_name)),
            }),
        }
    }
}

/// Adjust lightness in HSL space.
fn adjust_lightness(colour: Colour, percent: f32) -> Colour {
    use palette::{Hsl, IntoColor, Srgb};

    let rgb: Srgb<f32> = Srgb::new(
        colour.r as f32 / 255.0,
        colour.g as f32 / 255.0,
        colour.b as f32 / 255.0,
    );

    let mut hsl: Hsl = rgb.into_color();

    // Adjust lightness by percentage (relative to remaining range)
    let delta = percent / 100.0;
    if delta > 0.0 {
        // Lighten: move toward 1.0
        hsl.lightness += (1.0 - hsl.lightness) * delta;
    } else {
        // Darken: move toward 0.0
        hsl.lightness += hsl.lightness * delta;
    }
    hsl.lightness = hsl.lightness.clamp(0.0, 1.0);

    let rgb_out: Srgb<f32> = hsl.into_color();
    Colour::new(
        (rgb_out.red * 255.0).round() as u8,
        (rgb_out.green * 255.0).round() as u8,
        (rgb_out.blue * 255.0).round() as u8,
        colour.a,
    )
}

/// Adjust saturation in HSL space.
fn adjust_saturation(colour: Colour, percent: f32) -> Colour {
    use palette::{Hsl, IntoColor, Srgb};

    let rgb: Srgb<f32> = Srgb::new(
        colour.r as f32 / 255.0,
        colour.g as f32 / 255.0,
        colour.b as f32 / 255.0,
    );

    let mut hsl: Hsl = rgb.into_color();

    // Adjust saturation by percentage (relative to remaining range)
    let delta = percent / 100.0;
    if delta > 0.0 {
        // Saturate: move toward 1.0
        hsl.saturation += (1.0 - hsl.saturation) * delta;
    } else {
        // Desaturate: move toward 0.0
        hsl.saturation += hsl.saturation * delta;
    }
    hsl.saturation = hsl.saturation.clamp(0.0, 1.0);

    let rgb_out: Srgb<f32> = hsl.into_color();
    Colour::new(
        (rgb_out.red * 255.0).round() as u8,
        (rgb_out.green * 255.0).round() as u8,
        (rgb_out.blue * 255.0).round() as u8,
        colour.a,
    )
}

/// Mix two colours by a factor (0.0 = first colour, 1.0 = second colour).
fn mix_colours(a: Colour, b: Colour, factor: f32) -> Colour {
    let factor = factor.clamp(0.0, 1.0);
    let inv = 1.0 - factor;

    Colour::new(
        ((a.r as f32 * inv) + (b.r as f32 * factor)).round() as u8,
        ((a.g as f32 * inv) + (b.g as f32 * factor)).round() as u8,
        ((a.b as f32 * inv) + (b.b as f32 * factor)).round() as u8,
        ((a.a as f32 * inv) + (b.a as f32 * factor)).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        let expr = ColourExpr::parse("#FF0000").unwrap();
        assert_eq!(expr, ColourExpr::Hex("#FF0000".to_string()));
    }

    #[test]
    fn test_parse_reference() {
        let expr = ColourExpr::parse("$gold").unwrap();
        assert_eq!(expr, ColourExpr::Reference("gold".to_string()));
    }

    #[test]
    fn test_parse_percent() {
        let expr = ColourExpr::parse("20%").unwrap();
        assert_eq!(expr, ColourExpr::Percent(20.0));

        let expr = ColourExpr::parse("50.5%").unwrap();
        assert_eq!(expr, ColourExpr::Percent(50.5));
    }

    #[test]
    fn test_parse_function() {
        let expr = ColourExpr::parse("darken($gold, 20%)").unwrap();
        assert_eq!(
            expr,
            ColourExpr::Function {
                name: "darken".to_string(),
                args: vec![
                    ColourExpr::Reference("gold".to_string()),
                    ColourExpr::Percent(20.0),
                ],
            }
        );
    }

    #[test]
    fn test_parse_nested_function() {
        let expr = ColourExpr::parse("darken(lighten($gold, 10%), 5%)").unwrap();
        match expr {
            ColourExpr::Function { name, args } => {
                assert_eq!(name, "darken");
                assert_eq!(args.len(), 2);
                match &args[0] {
                    ColourExpr::Function {
                        name: inner_name,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_name, "lighten");
                        assert_eq!(inner_args.len(), 2);
                    }
                    _ => panic!("Expected nested function"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_mix() {
        let expr = ColourExpr::parse("mix($a, $b, 50%)").unwrap();
        assert_eq!(
            expr,
            ColourExpr::Function {
                name: "mix".to_string(),
                args: vec![
                    ColourExpr::Reference("a".to_string()),
                    ColourExpr::Reference("b".to_string()),
                    ColourExpr::Percent(50.0),
                ],
            }
        );
    }

    #[test]
    fn test_eval_hex() {
        let evaluator = ExprEvaluator::new(|_| None);
        let expr = ColourExpr::parse("#FF0000").unwrap();
        let colour = evaluator.eval(&expr).unwrap();
        assert_eq!(colour, Colour::rgb(255, 0, 0));
    }

    #[test]
    fn test_eval_reference() {
        let evaluator = ExprEvaluator::new(|name| {
            if name == "gold" {
                Some(Colour::rgb(247, 173, 69))
            } else {
                None
            }
        });

        let expr = ColourExpr::parse("$gold").unwrap();
        let colour = evaluator.eval(&expr).unwrap();
        assert_eq!(colour, Colour::rgb(247, 173, 69));
    }

    #[test]
    fn test_eval_darken() {
        let evaluator = ExprEvaluator::new(|name| {
            if name == "white" {
                Some(Colour::WHITE)
            } else {
                None
            }
        });

        let expr = ColourExpr::parse("darken($white, 50%)").unwrap();
        let colour = evaluator.eval(&expr).unwrap();

        // White darkened by 50% should be grey
        assert!(colour.r < 255);
        assert!(colour.g < 255);
        assert!(colour.b < 255);
        assert_eq!(colour.r, colour.g);
        assert_eq!(colour.g, colour.b);
    }

    #[test]
    fn test_eval_lighten() {
        let evaluator = ExprEvaluator::new(|name| {
            if name == "black" {
                Some(Colour::BLACK)
            } else {
                None
            }
        });

        let expr = ColourExpr::parse("lighten($black, 50%)").unwrap();
        let colour = evaluator.eval(&expr).unwrap();

        // Black lightened by 50% should be grey
        assert!(colour.r > 0);
        assert!(colour.g > 0);
        assert!(colour.b > 0);
    }

    #[test]
    fn test_eval_mix() {
        let evaluator = ExprEvaluator::new(|name| match name {
            "black" => Some(Colour::BLACK),
            "white" => Some(Colour::WHITE),
            _ => None,
        });

        let expr = ColourExpr::parse("mix($black, $white, 50%)").unwrap();
        let colour = evaluator.eval(&expr).unwrap();

        // 50% mix of black and white should be grey
        assert_eq!(colour.r, 128);
        assert_eq!(colour.g, 128);
        assert_eq!(colour.b, 128);
    }

    #[test]
    fn test_eval_alpha() {
        let evaluator = ExprEvaluator::new(|name| {
            if name == "red" {
                Some(Colour::rgb(255, 0, 0))
            } else {
                None
            }
        });

        let expr = ColourExpr::parse("alpha($red, 50%)").unwrap();
        let colour = evaluator.eval(&expr).unwrap();

        assert_eq!(colour.r, 255);
        assert_eq!(colour.g, 0);
        assert_eq!(colour.b, 0);
        // 50% of 255 = 127.5, rounds to 127 or 128 depending on impl
        assert!(colour.a == 127 || colour.a == 128);
    }

    #[test]
    fn test_eval_nested() {
        let evaluator = ExprEvaluator::new(|name| {
            if name == "grey" {
                Some(Colour::rgb(128, 128, 128))
            } else {
                None
            }
        });

        // Lighten then darken should return close to original
        let expr = ColourExpr::parse("darken(lighten($grey, 20%), 20%)").unwrap();
        let colour = evaluator.eval(&expr).unwrap();

        // Should be close to 128 (some rounding errors expected)
        assert!((colour.r as i32 - 128).abs() < 10);
    }

    #[test]
    fn test_saturate_desaturate() {
        let evaluator = ExprEvaluator::new(|name| {
            if name == "red" {
                Some(Colour::rgb(255, 100, 100))
            } else {
                None
            }
        });

        let expr = ColourExpr::parse("desaturate($red, 100%)").unwrap();
        let colour = evaluator.eval(&expr).unwrap();

        // Fully desaturated should be greyscale
        assert_eq!(colour.r, colour.g);
        assert_eq!(colour.g, colour.b);
    }

    #[test]
    fn test_unknown_function() {
        let evaluator = ExprEvaluator::new(|_| None);
        let expr = ColourExpr::parse("unknown($x, 20%)").unwrap();
        assert!(evaluator.eval(&expr).is_err());
    }

    #[test]
    fn test_undefined_reference() {
        let evaluator = ExprEvaluator::new(|_| None);
        let expr = ColourExpr::parse("$undefined").unwrap();
        assert!(evaluator.eval(&expr).is_err());
    }
}
