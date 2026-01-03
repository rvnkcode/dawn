use crate::Handler;
use clap::{Args, Parser, Subcommand};
use dawn::domain::task::port::TaskService;

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, subcommand_precedence_over_arg = true, version)]
pub struct Cli {
    filters: Vec<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add(Modification),
    All(Modification),
    Modify(Modification),
    Done(Modification),
}

#[derive(Args)]
pub struct Modification {
    pub description: Vec<String>,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn handle_command(&self, task_service: impl TaskService) -> anyhow::Result<()> {
        let handler = Handler::new(task_service);

        match &self.command {
            Some(Commands::Add(modification)) => handler.add(&self.filters, modification)?,
            Some(Commands::All(modification)) => handler.all(&self.filters, modification)?,
            Some(Commands::Modify(modification)) => handler.modify(&self.filters, modification)?,
            Some(Commands::Done(modification)) => handler.done(&self.filters, modification)?,
            None => handler.next(&self.filters)?,
        }
        Ok(())
    }
}
