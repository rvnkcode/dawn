use dawn_cli::Cli;

fn main() -> anyhow::Result<()> {
    dawn::bootstrap()?;
    let cli = Cli::new();
    cli.handle_command()
}
