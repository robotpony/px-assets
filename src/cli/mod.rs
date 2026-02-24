pub mod build;
pub mod init;
pub mod palette;
pub mod validate;

use clap::{Parser, Subcommand};

/// px - Sprite and map pipeline generator
#[derive(Parser, Debug)]
#[command(name = "px")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build sprites and maps from definition files
    Build(build::BuildArgs),

    /// Initialize a px project (generates px.yaml)
    Init(init::InitArgs),

    /// Extract a colour palette from a PNG file
    Palette(palette::PaletteArgs),

    /// Validate definition files without rendering
    Validate(validate::ValidateArgs),
}
