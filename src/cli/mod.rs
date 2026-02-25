pub mod build;
pub mod completions;
pub mod init;
pub mod list;
pub mod palette;
pub mod validate;

use clap::{Parser, Subcommand};

use crate::output::Verbosity;

/// px - Sprite and map pipeline generator
#[derive(Parser, Debug)]
#[command(name = "px")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output (extra detail)
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Suppress status output (errors and summary only)
    #[arg(long, short, global = true)]
    pub quiet: bool,
}

impl Cli {
    /// Resolve verbose/quiet flags into a Verbosity level.
    pub fn verbosity(&self) -> Verbosity {
        if self.quiet {
            Verbosity::Quiet
        } else if self.verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build sprites and maps from definition files
    Build(build::BuildArgs),

    /// Generate shell completions
    Completions(completions::CompletionsArgs),

    /// Initialize a px project (generates px.yaml)
    Init(init::InitArgs),

    /// List discovered assets
    List(list::ListArgs),

    /// Extract a colour palette from a PNG file
    Palette(palette::PaletteArgs),

    /// Validate definition files without rendering
    Validate(validate::ValidateArgs),
}
