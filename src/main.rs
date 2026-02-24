use clap::Parser;
use miette::Result;
use px::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => px::cli::build::run(args)?,
        Commands::Init(args) => px::cli::init::run(args)?,
        Commands::Palette(args) => px::cli::palette::run(args)?,
        Commands::Validate(args) => px::cli::validate::run(args)?,
    }

    Ok(())
}
