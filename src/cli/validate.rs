use std::path::PathBuf;
use std::process;

use clap::Args;

use crate::discovery::{discover_paths, LoadOptions};
use crate::error::Result;
use crate::validation::{print_diagnostics, validate_registry};

/// Validate definition files without rendering
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Files or directories to validate
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

pub fn run(args: ValidateArgs) -> Result<()> {
    // Discover and load assets
    let discovery = discover_paths(&args.files)?;
    let total = discovery.scan.total();
    eprintln!("Validating {} asset(s)...", total);

    let builder = crate::discovery::load_assets(&discovery.scan, &LoadOptions::with_builtins())?;
    let registry = builder.build()?;

    // Run validation checks
    let result = validate_registry(&registry);
    print_diagnostics(&result);

    if result.has_errors() {
        process::exit(1);
    }

    Ok(())
}
