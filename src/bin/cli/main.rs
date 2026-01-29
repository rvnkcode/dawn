use clap::Parser;
use dawn::inbound::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.handle_command()
}
