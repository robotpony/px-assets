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
                            .with_help(format!(
                                "Define it in a .stamp.md file:\n  ---\n  name: {}\n  glyph: {}\n  ---\nOr use a builtin: corner, edge-h, edge-v, solid, fill, transparent",
                                name, glyph
                            )),
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
                            .with_help(format!(
                                "Define it in a .brush.md file, or use a builtin: solid, checker, diagonal-l, diagonal-r, h-line, v-line, noise"
                            )),
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

/// Check that all targets use a supported format.
pub fn check_target_format(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for target in registry.targets() {
        if target.format != "png" && target.format != "p8" {
            result.push(
                Diagnostic::warning(
                    "px::validate::unsupported-target-format",
                    format!(
                        "Target '{}' uses format '{}' which is not yet supported",
                        target.name, target.format
                    ),
                )
                .with_help("Supported formats: 'png', 'p8'"),
            );
        }
    }

    result
}

/// Check for assets that are never referenced by any shape, prefab, or map.
pub fn check_unused_assets(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Collect all referenced stamp/brush names from shape legends
    let mut used_stamps: HashSet<String> = HashSet::new();
    let mut used_brushes: HashSet<String> = HashSet::new();
    let mut used_shapes: HashSet<String> = HashSet::new();
    let mut used_palettes: HashSet<String> = HashSet::new();
    let _used_shaders: HashSet<String> = HashSet::new();

    for shape in registry.shapes() {
        for entry in shape.legend().values() {
            match entry {
                LegendEntry::StampRef(name) => {
                    used_stamps.insert(name.clone());
                }
                LegendEntry::BrushRef { name, .. } | LegendEntry::Fill { name, .. } => {
                    used_brushes.insert(name.clone());
                }
            }
        }
    }

    // Collect shape/prefab references from prefabs and maps
    for prefab in registry.prefabs() {
        for name in prefab.referenced_names() {
            used_shapes.insert(name.to_string());
        }
    }
    for map in registry.maps() {
        for name in map.referenced_names() {
            if name != "empty" {
                used_shapes.insert(name.to_string());
            }
        }
    }

    // Collect palette references from shaders
    for shader in registry.shaders() {
        used_palettes.insert(shader.palette.clone());
    }

    // Collect shader references (from being named in shapes/builds -- we can't
    // know which shader is used at build time, so skip shader usage check)

    // Builtin names to skip
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

    // Check user-defined stamps
    for name in registry.stamp_names() {
        if builtin_stamp_names.contains(name) {
            continue;
        }
        if !used_stamps.contains(name) {
            result.push(
                Diagnostic::warning(
                    "px::validate::unused-asset",
                    format!("Stamp '{}' is never referenced in any shape legend", name),
                )
                .with_help("Remove the unused stamp or reference it in a shape legend"),
            );
        }
    }

    // Check user-defined brushes
    for name in registry.brush_names() {
        if builtin_brush_names.contains(name) {
            continue;
        }
        if !used_brushes.contains(name) {
            result.push(
                Diagnostic::warning(
                    "px::validate::unused-asset",
                    format!("Brush '{}' is never referenced in any shape legend", name),
                )
                .with_help("Remove the unused brush or reference it in a shape legend"),
            );
        }
    }

    // Check palettes (skip "default")
    for name in registry.palette_names() {
        if name == "default" {
            continue;
        }
        if !used_palettes.contains(name) {
            result.push(
                Diagnostic::warning(
                    "px::validate::unused-asset",
                    format!("Palette '{}' is never referenced by any shader", name),
                )
                .with_help("Remove the unused palette or reference it in a shader"),
            );
        }
    }

    // Check shapes (only those not referenced by any prefab or map)
    for name in registry.shape_names() {
        if !used_shapes.contains(name) {
            // Don't warn if there are no prefabs or maps (shapes are the leaf output)
            if registry.prefabs().next().is_some() || registry.maps().next().is_some() {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unused-asset",
                        format!("Shape '{}' is never referenced in any prefab or map", name),
                    )
                    .with_help("Remove the unused shape or reference it in a prefab or map legend"),
                );
            }
        }
    }

    result
}

/// Check for user-defined stamps or brushes that shadow builtin definitions.
pub fn check_shadowed_definitions(registry: &AssetRegistry) -> ValidationResult {
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

    // User stamps in the registry that shadow builtins.
    // Since builtins are also loaded into the registry, we check if a user file
    // defined a stamp with the same name. The registry stores the last-added,
    // so if the user's stamp replaced a builtin, the registry has the user version.
    // We detect this by checking if the registry has a stamp whose name matches
    // a builtin AND the registry was built with builtins loaded (which it always
    // is in practice). The stamp in the registry is the user's version.
    for name in registry.stamp_names() {
        if builtin_stamp_names.contains(name) {
            // Check if user defined a stamp with this name.
            // Since builtins are loaded first and user stamps override,
            // any builtin-named stamp in the registry IS the user version.
            // We detect shadowing by comparing the stamp against the builtin.
            if let (Some(registry_stamp), Some(builtin_stamp)) = (
                registry.get_stamp(name),
                BuiltinStamps::get(name),
            ) {
                // If the stamp in the registry differs from the builtin, it's a shadow
                if registry_stamp.pixels() != builtin_stamp.pixels() {
                    result.push(
                        Diagnostic::warning(
                            "px::validate::shadowed-builtin",
                            format!("Stamp '{}' shadows a builtin stamp", name),
                        )
                        .with_help("Rename the stamp to avoid shadowing the builtin, or use this intentionally to override it"),
                    );
                }
            }
        }
    }

    for name in registry.brush_names() {
        if builtin_brush_names.contains(name) {
            if let (Some(registry_brush), Some(builtin_brush)) = (
                registry.get_brush(name),
                BuiltinBrushes::get(name),
            ) {
                if registry_brush.pattern() != builtin_brush.pattern() {
                    result.push(
                        Diagnostic::warning(
                            "px::validate::shadowed-builtin",
                            format!("Brush '{}' shadows a builtin brush", name),
                        )
                        .with_help("Rename the brush to avoid shadowing the builtin, or use this intentionally to override it"),
                    );
                }
            }
        }
    }

    result
}

/// Check for palette colours that are never referenced in any shape or shader.
pub fn check_unused_palette_colours(registry: &AssetRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Collect all colour references from shape brush bindings
    let mut used_colours: HashSet<String> = HashSet::new();

    // $edge and $fill are always implicitly used by semantic stamp tokens
    used_colours.insert("$edge".to_string());
    used_colours.insert("$fill".to_string());
    used_colours.insert("edge".to_string());
    used_colours.insert("fill".to_string());

    // Collect colour refs from shape legends (brush bindings)
    for shape in registry.shapes() {
        for entry in shape.legend().values() {
            let bindings = match entry {
                LegendEntry::BrushRef { bindings, .. } => bindings,
                LegendEntry::Fill { bindings, .. } => bindings,
                _ => continue,
            };
            for colour_ref in bindings.values() {
                used_colours.insert(colour_ref.clone());
                // Also add without $ prefix
                if let Some(stripped) = colour_ref.strip_prefix('$') {
                    used_colours.insert(stripped.to_string());
                }
            }
        }
    }

    // Collect colour refs from shader palette_variant
    for shader in registry.shaders() {
        if let Some(variant) = &shader.palette_variant {
            used_colours.insert(variant.clone());
        }
    }

    // Check each palette's colours
    for palette in registry.palettes() {
        if palette.name == "default" {
            continue;
        }
        for colour_name in palette.colour_names() {
            if !used_colours.contains(colour_name) && !used_colours.contains(&format!("${}", colour_name)) {
                result.push(
                    Diagnostic::warning(
                        "px::validate::unused-colour",
                        format!(
                            "Palette '{}': colour '{}' is never referenced",
                            palette.name, colour_name
                        ),
                    )
                    .with_help("Remove the unused colour or reference it in a shape legend binding"),
                );
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::RegistryBuilder;
    use crate::types::{Brush, Map, Palette, PaletteBuilder, Prefab, Shader, Shape, Stamp, PixelToken, Target};
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

    #[test]
    fn test_check_target_format_png_ok() {
        let mut builder = RegistryBuilder::new();
        builder.add_target(Target::new("web", "png"));
        let registry = build_registry(builder);

        let result = check_target_format(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_target_format_p8_ok() {
        let mut builder = RegistryBuilder::new();
        builder.add_target(Target::new("pico", "p8"));
        let registry = build_registry(builder);

        let result = check_target_format(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_target_format_unsupported() {
        let mut builder = RegistryBuilder::new();
        builder.add_target(Target::new("bmp-out", "bmp"));
        let registry = build_registry(builder);

        let result = check_target_format(&registry);
        assert!(result.has_warnings());
    }

    // -- check_unused_assets --

    #[test]
    fn test_check_unused_assets_stamp_used() {
        let mut builder = RegistryBuilder::new();
        builder.add_stamp(Stamp::single("brick", Some('B'), PixelToken::Edge));

        let mut legend = HashMap::new();
        legend.insert('B', LegendEntry::StampRef("brick".to_string()));
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['B']], legend));
        let registry = build_registry(builder);

        let result = check_unused_assets(&registry);
        // brick is used in wall's legend, so no warning for it
        let stamp_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Stamp 'brick'"))
            .collect();
        assert!(stamp_warnings.is_empty());
    }

    #[test]
    fn test_check_unused_assets_stamp_unused() {
        let mut builder = RegistryBuilder::new();
        builder.add_stamp(Stamp::single("orphan", Some('O'), PixelToken::Edge));
        // Shape that doesn't reference orphan
        builder.add_shape(Shape::new("test", vec![], vec![vec!['#']], HashMap::new()));
        let registry = build_registry(builder);

        let result = check_unused_assets(&registry);
        let stamp_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Stamp 'orphan'"))
            .collect();
        assert_eq!(stamp_warnings.len(), 1);
    }

    #[test]
    fn test_check_unused_assets_shape_unreferenced() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("lonely", vec![], vec![vec!['#']], HashMap::new()));
        // Add a map so the check activates (only warns when prefabs/maps exist)
        let mut legend = HashMap::new();
        legend.insert('.', "empty".to_string());
        builder.add_map(Map::new("level", vec![], vec![vec!['.']], legend));
        let registry = build_registry(builder);

        let result = check_unused_assets(&registry);
        let shape_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Shape 'lonely'"))
            .collect();
        assert_eq!(shape_warnings.len(), 1);
    }

    #[test]
    fn test_check_unused_assets_palette_unreferenced() {
        let mut pb = PaletteBuilder::new("orphan-palette");
        pb.define("red", "#FF0000");
        let palette = pb.build(None).unwrap();

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        let registry = build_registry(builder);

        let result = check_unused_assets(&registry);
        let palette_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Palette 'orphan-palette'"))
            .collect();
        assert_eq!(palette_warnings.len(), 1);
    }

    #[test]
    fn test_check_unused_assets_palette_referenced_by_shader() {
        let mut pb = PaletteBuilder::new("game");
        pb.define("red", "#FF0000");
        let palette = pb.build(None).unwrap();

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        builder.add_shader(Shader::new("main", "game"));
        let registry = build_registry(builder);

        let result = check_unused_assets(&registry);
        let palette_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Palette 'game'"))
            .collect();
        assert!(palette_warnings.is_empty());
    }

    // -- check_shadowed_definitions --

    #[test]
    fn test_check_shadowed_definitions_no_shadow() {
        let mut builder = RegistryBuilder::new();
        // User stamp with a unique name (not a builtin name)
        builder.add_stamp(Stamp::single("brick", Some('B'), PixelToken::Edge));
        let registry = build_registry(builder);

        let result = check_shadowed_definitions(&registry);
        let shadow_warnings: Vec<_> = result.iter()
            .filter(|d| d.code == "px::validate::shadowed-builtin")
            .collect();
        assert!(shadow_warnings.is_empty());
    }

    #[test]
    fn test_check_shadowed_definitions_stamp_shadows_builtin() {
        let mut builder = RegistryBuilder::new();
        // "solid" is a builtin stamp name; override it with different pixels
        builder.add_stamp(Stamp::new(
            "solid",
            Some('#'),
            vec![vec![PixelToken::Fill]], // builtin solid uses Edge, this uses Fill
        ));
        let registry = build_registry(builder);

        let result = check_shadowed_definitions(&registry);
        let shadow_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Stamp 'solid' shadows"))
            .collect();
        assert_eq!(shadow_warnings.len(), 1);
    }

    #[test]
    fn test_check_shadowed_definitions_brush_shadows_builtin() {
        let mut builder = RegistryBuilder::new();
        // "checker" is a builtin brush; override with a different pattern
        builder.add_brush(Brush::new("checker", vec![vec!['A']]));
        let registry = build_registry(builder);

        let result = check_shadowed_definitions(&registry);
        let shadow_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("Brush 'checker' shadows"))
            .collect();
        assert_eq!(shadow_warnings.len(), 1);
    }

    // -- check_unused_palette_colours --

    #[test]
    fn test_check_unused_palette_colours_edge_fill_always_used() {
        let mut pb = PaletteBuilder::new("game");
        pb.define("edge", "#000000");
        pb.define("fill", "#FFFFFF");
        let palette = pb.build(None).unwrap();

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        let registry = build_registry(builder);

        let result = check_unused_palette_colours(&registry);
        // edge and fill are implicitly used by stamp tokens
        let colour_warnings: Vec<_> = result.iter()
            .filter(|d| d.code == "px::validate::unused-colour")
            .collect();
        assert!(colour_warnings.is_empty());
    }

    #[test]
    fn test_check_unused_palette_colours_unused() {
        let mut pb = PaletteBuilder::new("game");
        pb.define("edge", "#000000");
        pb.define("fill", "#FFFFFF");
        pb.define("accent", "#FF0000");
        let palette = pb.build(None).unwrap();

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        let registry = build_registry(builder);

        let result = check_unused_palette_colours(&registry);
        let colour_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("'accent'"))
            .collect();
        assert_eq!(colour_warnings.len(), 1);
    }

    #[test]
    fn test_check_unused_palette_colours_used_in_binding() {
        let mut pb = PaletteBuilder::new("game");
        pb.define("edge", "#000000");
        pb.define("fill", "#FFFFFF");
        pb.define("accent", "#FF0000");
        let palette = pb.build(None).unwrap();

        let mut bindings = HashMap::new();
        bindings.insert('A', "$accent".to_string());
        let mut legend = HashMap::new();
        legend.insert('~', LegendEntry::Fill {
            name: "checker".to_string(),
            bindings,
        });

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        builder.add_shape(Shape::new("test", vec![], vec![vec!['~']], legend));
        let registry = build_registry(builder);

        let result = check_unused_palette_colours(&registry);
        let colour_warnings: Vec<_> = result.iter()
            .filter(|d| d.message.contains("'accent'"))
            .collect();
        assert!(colour_warnings.is_empty());
    }

    // -- check_palette_refs --

    #[test]
    fn test_check_palette_refs_valid() {
        let mut pb = PaletteBuilder::new("game");
        pb.define("dark", "#222222");
        let palette = pb.build(None).unwrap();

        let mut bindings = HashMap::new();
        bindings.insert('A', "$dark".to_string());
        let mut legend = HashMap::new();
        legend.insert('~', LegendEntry::Fill {
            name: "checker".to_string(),
            bindings,
        });

        let mut builder = RegistryBuilder::new();
        builder.add_palette(palette);
        builder.add_shape(Shape::new("test", vec![], vec![vec!['~']], legend));
        let registry = build_registry(builder);

        let result = check_palette_refs(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_palette_refs_missing() {
        let mut bindings = HashMap::new();
        bindings.insert('A', "$nonexistent".to_string());
        let mut legend = HashMap::new();
        legend.insert('~', LegendEntry::Fill {
            name: "checker".to_string(),
            bindings,
        });

        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("test", vec![], vec![vec!['~']], legend));
        let registry = build_registry(builder);

        let result = check_palette_refs(&registry);
        assert!(result.has_warnings());
    }

    // -- check_duplicate_names --

    #[test]
    fn test_check_duplicate_names_shape_prefab_collision() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("widget", vec![], vec![vec!['#']], HashMap::new()));

        let mut legend = HashMap::new();
        legend.insert('W', "widget".to_string());
        builder.add_prefab(Prefab::new("widget", vec![], vec![vec!['W']], legend));
        let registry = build_registry(builder);

        let result = check_duplicate_names(&registry);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_check_duplicate_names_no_collision() {
        let mut builder = RegistryBuilder::new();
        builder.add_shape(Shape::new("wall", vec![], vec![vec!['#']], HashMap::new()));

        let mut legend = HashMap::new();
        legend.insert('W', "wall".to_string());
        builder.add_prefab(Prefab::new("tower", vec![], vec![vec!['W']], legend));
        let registry = build_registry(builder);

        let result = check_duplicate_names(&registry);
        assert!(!result.has_warnings());
    }
}
