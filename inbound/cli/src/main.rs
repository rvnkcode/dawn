use dawn_cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::new();
    cli.handle_command()
}
