use clap::Parser;
use dawn::inbound::cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.handle_command()
}
