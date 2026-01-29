use clap::Parser;

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, version)]
pub struct Cli;

impl Cli {
    pub fn handle_command(&self) -> anyhow::Result<()> {
        println!("{}", crate::greet());
        Ok(())
    }
}
