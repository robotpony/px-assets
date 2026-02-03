use clap::Parser;
use miette::Result;
use px::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => px::cli::build::run(args)?,
        Commands::Validate(args) => px::cli::validate::run(args)?,
    }

    Ok(())
}
