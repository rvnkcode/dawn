use clap::Parser;

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, subcommand_precedence_over_arg = true, version)]
pub struct Cli {}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn handle_command(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
