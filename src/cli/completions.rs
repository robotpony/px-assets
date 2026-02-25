//! Shell completions generation.

use clap::Args;
use clap_complete::Shell;

/// Generate shell completions
#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs) -> crate::error::Result<()> {
    let mut cmd = <super::Cli as clap::CommandFactory>::command();
    clap_complete::generate(args.shell, &mut cmd, "px", &mut std::io::stdout());
    Ok(())
}
