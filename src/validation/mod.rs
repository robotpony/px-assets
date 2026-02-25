//! Validation system for px asset registries.
//!
//! Runs a suite of checks against a built registry and reports errors
//! and warnings. Used by both `px validate` and `px build --validate`.

mod checks;
mod warning;

pub use warning::{Diagnostic, Severity, ValidationResult};

use crate::output::{plural, Printer};
use crate::registry::AssetRegistry;

/// Run all validation checks against the registry.
pub fn validate_registry(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    result.merge(checks::check_empty_grids(registry));
    result.merge(checks::check_duplicate_names(registry));
    result.merge(checks::check_shape_legend_refs(registry));
    result.merge(checks::check_prefab_legend_refs(registry));
    result.merge(checks::check_map_legend_refs(registry));
    result.merge(checks::check_unmapped_glyphs(registry));
    result.merge(checks::check_unused_legends(registry));
    result.merge(checks::check_stamp_sizes(registry));
    result.merge(checks::check_palette_refs(registry));
    result.merge(checks::check_target_format(registry));
    result.merge(checks::check_unused_assets(registry));
    result.merge(checks::check_shadowed_definitions(registry));
    result.merge(checks::check_unused_palette_colours(registry));

    result
}

/// Print diagnostics to stderr with coloured output.
pub fn print_diagnostics(result: &ValidationResult, printer: &Printer) {
    for d in result.iter() {
        let is_error = matches!(d.severity, Severity::Error);
        let label = if is_error { "error" } else { "warning" };
        let severity = printer.severity(label, is_error);
        let code = printer.dim(&format!("[{}]", d.code));
        eprintln!("      {}{}: {}", severity, code, d.message);
        if let Some(help) = &d.help {
            eprintln!("             {}: {}", printer.cyan("help"), help);
        }
    }

    let errors = result.error_count();
    let warnings = result.warning_count();

    if errors > 0 {
        printer.error(
            "Failed",
            &format!("{}, {}", plural(errors, "error", "errors"), plural(warnings, "warning", "warnings")),
        );
    } else if warnings > 0 {
        printer.success(
            "Passed",
            &format!("({})", plural(warnings, "warning", "warnings")),
        );
    } else {
        printer.success("Passed", "all clear");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::RegistryBuilder;
    use crate::types::{LegendEntry, Shape};
    use std::collections::HashMap;

    #[test]
    fn test_validate_empty_registry() {
        let registry = RegistryBuilder::new().build().unwrap();
        let result = validate_registry(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_valid_shape() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new(
            "test",
            vec![],
            vec![vec!['#', '.']],
            HashMap::new(),
        ));
        let registry = builder.build().unwrap();

        let result = validate_registry(&registry);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_validate_catches_missing_ref() {
        let mut legend = HashMap::new();
        legend.insert('B', LegendEntry::StampRef("nonexistent".to_string()));

        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['B']], legend));
        let registry = builder.build().unwrap();

        let result = validate_registry(&registry);
        assert!(result.has_errors());
    }
}
