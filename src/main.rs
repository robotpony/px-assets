use clap::Parser;
use miette::Result;
use px::cli::{Cli, Commands};
use px::output::Printer;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let printer = Printer::with_verbosity(cli.verbosity());

    match cli.command {
        Commands::Build(args) => px::cli::build::run(args, &printer)?,
        Commands::Completions(args) => px::cli::completions::run(args)?,
        Commands::Init(args) => px::cli::init::run(args, &printer)?,
        Commands::List(args) => px::cli::list::run(args, &printer)?,
        Commands::Palette(args) => px::cli::palette::run(args, &printer)?,
        Commands::Validate(args) => px::cli::validate::run(args, &printer)?,
    }

    Ok(())
}
