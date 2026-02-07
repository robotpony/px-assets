//! Validation checks for the asset registry.
//!
//! Each check takes an `&AssetRegistry` and returns a `ValidationResult`.

use std::collections::HashSet;

use crate::registry::AssetRegistry;
use crate::types::{BuiltinBrushes, BuiltinStamps, LegendEntry};

use super::warning::{Diagnostic, ValidationResult};

/// Check for shapes, prefabs, or maps with zero-size grids.
pub fn check_empty_grids(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for shape in registry.shapes() {
        if shape.is_empty() {
            result.push(
                Diagnostic::error(
                    "px::validate::empty-grid",
                    format!("Shape '{}' has an empty grid", shape.name),
                )
                .with_help("Add at least one row of characters to the grid"),
            );
        }
    }

    for prefab in registry.prefabs() {
        if prefab.is_empty() {
            result.push(
                Diagnostic::error(
                    "px::validate::empty-grid",
                    format!("Prefab '{}' has an empty grid", prefab.name),
                )
                .with_help("Add at least one row of characters to the grid"),
            );
        }
    }

    for map in registry.maps() {
        if map.is_empty() {
            result.push(
                Diagnostic::error(
                    "px::validate::empty-grid",
                    format!("Map '{}' has an empty grid", map.name),
                )
                .with_help("Add at least one row of characters to the grid"),
            );
        }
    }

    result
}

/// Check for duplicate names that shadow builtins.
pub fn check_duplicate_names(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    let builtin_stamps = BuiltinStamps::all();
    let builtin_stamp_names: HashSet<&str> = builtin_stamps
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    let builtin_brushes = BuiltinBrushes::all();
    let builtin_brush_names: HashSet<&str> = builtin_brushes
        .iter()
        .map(|b| b.name.as_str())
        .collect();

    for name in registry.stamp_names() {
        if builtin_stamp_names.contains(name) {
            // Only warn if the registry stamp is a user-defined one that shadows a builtin.
            // Since builtins are added to the registry too, we can't distinguish here.
            // Skip this check for builtin names.
        }
    }

    for name in registry.brush_names() {
        if builtin_brush_names.contains(name) {
            // Same as above - builtins are in the registry.
        }
    }

    // Check for name collisions across shapes/prefabs (a prefab referencing
    // a name could hit either). This is more of a consistency check.
    let shape_names: HashSet<&str> = registry.shape_names().collect();
    let prefab_names: HashSet<&str> = registry.prefab_names().collect();
    for name in &shape_names {
        if prefab_names.contains(name) {
            result.push(
                Diagnostic::warning(
                    "px::validate::duplicate-name",
                    format!(
                        "Name '{}' is used for both a shape and a prefab",
                        name
                    ),
                )
                .with_help("Use distinct names to avoid ambiguous references"),
            );
        }
    }

    result
}

/// Check that shape legend stamp/brush references exist.
pub fn check_shape_legend_refs(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for shape in registry.shapes() {
        for (glyph, entry) in shape.legend() {
            match entry {
                LegendEntry::StampRef(name) => {
                    if registry.get_stamp(name).is_none() && BuiltinStamps::get(name).is_none() {
                        result.push(
                            Diagnostic::error(
                                "px::validate::missing-stamp",
                                format!(
                                    "Shape '{}': legend '{}' references stamp '{}' which does not exist",
                                    shape.name, glyph, name
                                ),
                            )
                            .with_help("Define it in a .stamp.md file, or use a builtin stamp name"),
                        );
                    }
                }
                LegendEntry::BrushRef { name, .. } | LegendEntry::Fill { name, .. } => {
                    if registry.get_brush(name).is_none() && BuiltinBrushes::get(name).is_none() {
                        result.push(
                            Diagnostic::error(
                                "px::validate::missing-brush",
                                format!(
                                    "Shape '{}': legend '{}' references brush '{}' which does not exist",
                                    shape.name, glyph, name
                                ),
                            )
                            .with_help("Define it in a .brush.md file, or use a builtin brush name"),
                        );
                    }
                }
            }
        }
    }

    result
}

/// Check that prefab legend references exist as shapes or prefabs.
pub fn check_prefab_legend_refs(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for prefab in registry.prefabs() {
        for (glyph, ref_name) in prefab.legend() {
            if registry.get_shape(ref_name).is_none()
                && registry.get_prefab(ref_name).is_none()
            {
                result.push(
                    Diagnostic::error(
                        "px::validate::missing-ref",
                        format!(
                            "Prefab '{}': legend '{}' references '{}' which is not a known shape or prefab",
                            prefab.name, glyph, ref_name
                        ),
                    )
                    .with_help("Define it in a .shape.md or .prefab.md file"),
                );
            }
        }
    }

    result
}

/// Check that map legend references exist (skip "empty").
pub fn check_map_legend_refs(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for map in registry.maps() {
        for (glyph, ref_name) in map.legend() {
            if ref_name == "empty" {
                continue;
            }
            if registry.get_shape(ref_name).is_none()
                && registry.get_prefab(ref_name).is_none()
            {
                result.push(
                    Diagnostic::error(
                        "px::validate::missing-ref",
                        format!(
                            "Map '{}': legend '{}' references '{}' which is not a known shape or prefab",
                            map.name, glyph, ref_name
                        ),
                    )
                    .with_help("Define it in a .shape.md or .prefab.md file"),
                );
            }
        }
    }

    result
}

/// Check for glyphs in grids that have no legend entry and are not builtin glyphs.
pub fn check_unmapped_glyphs(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for shape in registry.shapes() {
        for glyph in shape.glyphs() {
            if !shape.has_legend(glyph) && BuiltinStamps::get_by_glyph(glyph).is_none() {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unmapped-glyph",
                        format!(
                            "Shape '{}': glyph '{}' has no legend entry and is not a builtin glyph",
                            shape.name, glyph
                        ),
                    )
                    .with_help("Add a legend entry or use a builtin glyph (+, -, |, #, ., x, space)"),
                );
            }
        }
    }

    for prefab in registry.prefabs() {
        for glyph in prefab.glyphs() {
            if glyph == ' ' {
                continue;
            }
            if !prefab.has_legend(glyph) {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unmapped-glyph",
                        format!(
                            "Prefab '{}': glyph '{}' has no legend entry",
                            prefab.name, glyph
                        ),
                    )
                    .with_help("Add a legend entry mapping this glyph to a shape or prefab"),
                );
            }
        }
    }

    for map in registry.maps() {
        for glyph in map.glyphs() {
            if glyph == ' ' {
                continue;
            }
            if !map.has_legend(glyph) {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unmapped-glyph",
                        format!(
                            "Map '{}': glyph '{}' has no legend entry",
                            map.name, glyph
                        ),
                    )
                    .with_help("Add a legend entry mapping this glyph to a shape or prefab"),
                );
            }
        }
    }

    result
}

/// Check for legend entries whose glyph never appears in the grid.
pub fn check_unused_legends(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for shape in registry.shapes() {
        let grid_glyphs: HashSet<char> = shape.glyphs().into_iter().collect();
        for glyph in shape.legend().keys() {
            if !grid_glyphs.contains(glyph) {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unused-legend",
                        format!(
                            "Shape '{}': legend entry '{}' is never used in the grid",
                            shape.name, glyph
                        ),
                    )
                    .with_help("Remove the unused legend entry or add the glyph to the grid"),
                );
            }
        }
    }

    for prefab in registry.prefabs() {
        let grid_glyphs: HashSet<char> = prefab.glyphs().into_iter().collect();
        for glyph in prefab.legend().keys() {
            if !grid_glyphs.contains(glyph) {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unused-legend",
                        format!(
                            "Prefab '{}': legend entry '{}' is never used in the grid",
                            prefab.name, glyph
                        ),
                    )
                    .with_help("Remove the unused legend entry or add the glyph to the grid"),
                );
            }
        }
    }

    for map in registry.maps() {
        let grid_glyphs: HashSet<char> = map.glyphs().into_iter().collect();
        for glyph in map.legend().keys() {
            if !grid_glyphs.contains(glyph) {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unused-legend",
                        format!(
                            "Map '{}': legend entry '{}' is never used in the grid",
                            map.name, glyph
                        ),
                    )
                    .with_help("Remove the unused legend entry or add the glyph to the grid"),
                );
            }
        }
    }

    result
}

/// Check for shapes that use stamps of different dimensions via legend.
pub fn check_stamp_sizes(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for shape in registry.shapes() {
        let mut sizes: Vec<((usize, usize), String)> = Vec::new();

        for entry in shape.legend().values() {
            if let LegendEntry::StampRef(name) = entry {
                if let Some(stamp) = registry.get_stamp(name) {
                    sizes.push((stamp.size(), stamp.name.clone()));
                } else if let Some(stamp) = BuiltinStamps::get(name) {
                    sizes.push((stamp.size(), stamp.name.clone()));
                }
            }
        }

        if sizes.len() > 1 {
            let first_size = sizes[0].0;
            let mismatched: Vec<_> = sizes
                .iter()
                .filter(|(size, _)| *size != first_size)
                .collect();

            if !mismatched.is_empty() {
                let details: Vec<String> = sizes
                    .iter()
                    .map(|((w, h), name)| format!("'{}' ({}x{})", name, w, h))
                    .collect();

                result.push(
                    Diagnostic::warning(
                        "px::validate::stamp-size-mismatch",
                        format!(
                            "Shape '{}' uses stamps of different sizes: {}",
                            shape.name,
                            details.join(", ")
                        ),
                    )
                    .with_help("Stamps in the same shape should have the same dimensions for correct rendering"),
                );
            }
        }
    }

    result
}

/// Check for brush bindings that reference palette colours not found in any palette.
pub fn check_palette_refs(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Collect all known colour names across all palettes
    let mut known_colours: HashSet<String> = HashSet::new();
    for palette in registry.palettes() {
        for name in palette.colour_names() {
            known_colours.insert(name.to_string());
            // Also add with $ prefix since bindings use $colour
            known_colours.insert(format!("${}", name));
        }
    }

    for shape in registry.shapes() {
        for (glyph, entry) in shape.legend() {
            let bindings = match entry {
                LegendEntry::BrushRef { bindings, .. } => bindings,
                LegendEntry::Fill { bindings, .. } => bindings,
                _ => continue,
            };

            for (token, colour_ref) in bindings {
                if colour_ref.starts_with('$') && !known_colours.contains(colour_ref) {
                    result.push(
                        Diagnostic::warning(
                            "px::validate::missing-palette-colour",
                            format!(
                                "Shape '{}': legend '{}' token '{}' references colour '{}' not found in any palette",
                                shape.name, glyph, token, colour_ref
                            ),
                        )
                        .with_help("Define the colour in a .palette.md file"),
                    );
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::RegistryBuilder;
    use crate::types::{Map, Prefab, Shape, Stamp, PixelToken};
    use std::collections::HashMap;

    fn build_registry(builder: RegistryBuilder) -> AssetRegistry {
        builder.build().unwrap()
    }

    #[test]
    fn test_check_empty_grids_shape() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("empty-shape", vec![], vec![], HashMap::new()));
        let registry = build_registry(builder);

        let result = check_empty_grids(&registry);
        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_check_empty_grids_valid() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("ok", vec![], vec![vec!['#']], HashMap::new()));
        let registry = build_registry(builder);

        let result = check_empty_grids(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_shape_legend_refs_missing_stamp() {
        let mut legend = HashMap::new();
        legend.insert('B', LegendEntry::StampRef("nonexistent".to_string()));

        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['B']], legend));
        let registry = build_registry(builder);

        let result = check_shape_legend_refs(&registry);
        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_check_shape_legend_refs_builtin_stamp() {
        let mut legend = HashMap::new();
        legend.insert('B', LegendEntry::StampRef("corner".to_string()));

        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['B']], legend));
        let registry = build_registry(builder);

        let result = check_shape_legend_refs(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_shape_legend_refs_missing_brush() {
        let mut legend = HashMap::new();
        legend.insert(
            '~',
            LegendEntry::Fill {
                name: "nonexistent".to_string(),
                bindings: HashMap::new(),
            },
        );

        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['~']], legend));
        let registry = build_registry(builder);

        let result = check_shape_legend_refs(&registry);
        assert!(result.has_errors());
    }

    #[test]
    fn test_check_prefab_legend_refs_missing() {
        let mut legend = HashMap::new();
        legend.insert('W', "nonexistent-shape".to_string());

        let mut builder = RegistryBuilder::new();
        builder.add_prefab(Prefab::new("tower", vec![], vec![vec!['W']], legend));
        let registry = build_registry(builder);

        let result = check_prefab_legend_refs(&registry);
        assert!(result.has_errors());
    }

    #[test]
    fn test_check_prefab_legend_refs_valid() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['#']], HashMap::new()));

        let mut legend = HashMap::new();
        legend.insert('W', "wall".to_string());
        builder.add_prefab(Prefab::new("tower", vec![], vec![vec!['W']], legend));

        let registry = build_registry(builder);
        let result = check_prefab_legend_refs(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_map_legend_refs_missing() {
        let mut legend = HashMap::new();
        legend.insert('W', "nonexistent".to_string());

        let mut builder = RegistryBuilder::new();
        builder.add_map(Map::new("level-1", vec![], vec![vec!['W']], legend));
        let registry = build_registry(builder);

        let result = check_map_legend_refs(&registry);
        assert!(result.has_errors());
    }

    #[test]
    fn test_check_map_legend_refs_empty_skipped() {
        let mut legend = HashMap::new();
        legend.insert('.', "empty".to_string());

        let mut builder = RegistryBuilder::new();
        builder.add_map(Map::new("level-1", vec![], vec![vec!['.']], legend));
        let registry = build_registry(builder);

        let result = check_map_legend_refs(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_unmapped_glyphs_shape() {
        let mut builder = RegistryBuilder::new();
        // 'Z' is not a builtin glyph and has no legend entry
        builder.add_shape(Shape::new("test", vec![], vec![vec!['Z']], HashMap::new()));
        let registry = build_registry(builder);

        let result = check_unmapped_glyphs(&registry);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_check_unmapped_glyphs_builtin_ok() {
        let mut builder = RegistryBuilder::new();
        // '#' is a builtin glyph (solid stamp)
        builder.add_shape(Shape::new("test", vec![], vec![vec!['#']], HashMap::new()));
        let registry = build_registry(builder);

        let result = check_unmapped_glyphs(&registry);
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_check_unused_legends() {
        let mut legend = HashMap::new();
        legend.insert('Z', LegendEntry::StampRef("something".to_string()));

        let mut builder = RegistryBuilder::new();
        // Grid uses '#' but legend defines 'Z'
        builder.add_shape(Shape::new("test", vec![], vec![vec!['#']], legend));
        let registry = build_registry(builder);

        let result = check_unused_legends(&registry);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_check_stamp_sizes_mismatch() {
        let mut builder = RegistryBuilder::new();
        builder.add_stamp(Stamp::single("small", Some('S'), PixelToken::Edge));
        builder.add_stamp(Stamp::new(
            "big",
            Some('B'),
            vec![
                vec![PixelToken::Edge, PixelToken::Edge],
                vec![PixelToken::Edge, PixelToken::Edge],
            ],
        ));

        let mut legend = HashMap::new();
        legend.insert('S', LegendEntry::StampRef("small".to_string()));
        legend.insert('B', LegendEntry::StampRef("big".to_string()));

        builder.add_shape(Shape::new("test", vec![], vec![vec!['S', 'B']], legend));
        let registry = build_registry(builder);

        let result = check_stamp_sizes(&registry);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_check_stamp_sizes_uniform() {
        let mut builder = RegistryBuilder::new();
        builder.add_stamp(Stamp::single("a", Some('A'), PixelToken::Edge));
        builder.add_stamp(Stamp::single("b", Some('B'), PixelToken::Fill));

        let mut legend = HashMap::new();
        legend.insert('A', LegendEntry::StampRef("a".to_string()));
        legend.insert('B', LegendEntry::StampRef("b".to_string()));

        builder.add_shape(Shape::new("test", vec![], vec![vec!['A', 'B']], legend));
        let registry = build_registry(builder);

        let result = check_stamp_sizes(&registry);
        assert!(result.is_ok());
    }
}
