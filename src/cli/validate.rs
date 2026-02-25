use std::path::PathBuf;
use std::process;

use clap::Args;

use crate::discovery::{discover_paths, LoadOptions};
use crate::error::Result;
use crate::output::{plural, Printer};
use crate::validation::{print_diagnostics, validate_registry};

/// Validate definition files without rendering
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Files or directories to validate
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

pub fn run(args: ValidateArgs, printer: &Printer) -> Result<()> {

    // Discover and load assets
    let discovery = discover_paths(&args.files)?;
    let total = discovery.scan.total();
    printer.status("Validating", &format!("{}...", plural(total, "asset", "assets")));

    let builder = crate::discovery::load_assets(&discovery.scan, &LoadOptions::with_builtins())?;
    let registry = builder.build()?;

    // Run validation checks
    let result = validate_registry(&registry);
    print_diagnostics(&result, &printer);

    if result.has_errors() {
        process::exit(1);
    }

    Ok(())
}
