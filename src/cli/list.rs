//! List command implementation.
//!
//! Discovers assets and prints an organized inventory.

use std::path::PathBuf;

use clap::Args;

use crate::discovery::{discover, discover_paths, load_assets, LoadOptions};
use crate::error::Result;
use crate::output::Printer;

/// List discovered assets
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Files or directories to scan (default: current directory)
    pub files: Vec<PathBuf>,

    /// Show dependency relationships
    #[arg(long)]
    pub deps: bool,
}

pub fn run(args: ListArgs, printer: &Printer) -> Result<()> {
    let discovery = if args.files.is_empty() {
        discover(".")?
    } else {
        discover_paths(&args.files)?
    };

    let builder = load_assets(&discovery.scan, &LoadOptions::with_builtins())?;
    let registry = builder.build()?;

    if args.deps {
        print_deps(&registry, printer);
    } else {
        print_inventory(&registry, printer);
    }

    Ok(())
}

fn print_inventory(registry: &crate::registry::AssetRegistry, printer: &Printer) {
    let groups: &[(&str, Vec<String>)] = &[
        ("Palettes", sorted_names(registry.palette_names())),
        ("Stamps", sorted_names(registry.stamp_names())),
        ("Brushes", sorted_names(registry.brush_names())),
        ("Shaders", sorted_names(registry.shader_names())),
        ("Shapes", sorted_names(registry.shape_names())),
        ("Prefabs", sorted_names(registry.prefab_names())),
        ("Maps", sorted_names(registry.map_names())),
        ("Targets", sorted_names(registry.target_names())),
    ];

    for (label, names) in groups {
        if names.is_empty() {
            continue;
        }
        printer.info(label, &names.join(", "));
    }
}

fn print_deps(registry: &crate::registry::AssetRegistry, printer: &Printer) {
    let graph = registry.graph();

    for id in registry.build_order() {
        let deps: Vec<String> = graph.dependencies_of(id).map(|d| d.name.clone()).collect();
        let kind = id.kind.name();

        if deps.is_empty() {
            printer.info(kind, &id.name);
        } else {
            let dep_list = deps.join(", ");
            printer.info(kind, &format!("{} {} {}", id.name, printer.dim("->"), dep_list));
        }
    }
}

fn sorted_names<'a>(iter: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut names: Vec<String> = iter.map(|s| s.to_string()).collect();
    names.sort();
    names
}
