//! Validation system for px asset registries.
//!
//! Runs a suite of checks against a built registry and reports errors
//! and warnings. Used by both `px validate` and `px build --validate`.

mod checks;
mod warning;

pub use warning::{Diagnostic, Severity, ValidationResult};

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

    result
}

/// Print diagnostics to stderr.
pub fn print_diagnostics(result: &ValidationResult) {
    for d in result.iter() {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        eprintln!("  {}[{}]: {}", severity, d.code, d.message);
        if let Some(help) = &d.help {
            eprintln!("    help: {}", help);
        }
    }

    let errors = result.error_count();
    let warnings = result.warning_count();

    if errors > 0 {
        eprintln!(
            "Validation failed: {} error(s), {} warning(s)",
            errors, warnings
        );
    } else if warnings > 0 {
        eprintln!("Validation passed ({} warning(s))", warnings);
    } else {
        eprintln!("Validation passed.");
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
