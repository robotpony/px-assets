use crate::error::Result;
use clap::Args;
use std::path::PathBuf;

/// Build sprites and maps from definition files
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Input files to process
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Shader to apply
    #[arg(long)]
    pub shader: Option<String>,

    /// Output target
    #[arg(long)]
    pub target: Option<String>,

    /// Output directory
    #[arg(long, short, default_value = "dist")]
    pub output: PathBuf,
}

pub fn run(args: BuildArgs) -> Result<()> {
    // Stub implementation - will be filled in during Phase 1.7+
    println!("Building {} file(s)...", args.files.len());
    for file in &args.files {
        println!("  {}", file.display());
    }
    if let Some(shader) = &args.shader {
        println!("Using shader: {}", shader);
    }
    if let Some(target) = &args.target {
        println!("Target: {}", target);
    }
    println!("Output directory: {}", args.output.display());
    Ok(())
}
