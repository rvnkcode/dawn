use clap::Parser;

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, version)]
pub struct Cli;

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn handle_command(&self) -> anyhow::Result<()> {
        println!("{}", dawn::greet());
        Ok(())
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}
