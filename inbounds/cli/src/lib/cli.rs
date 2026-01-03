use crate::{Handler, handler::Status};
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
    /// Adds a new task
    Add(Modification),
    /// All tasks
    All(Modification),
    /// Modifies the existing task with provided arguments
    Modify(Modification),
    /// Marks the specified task as completed
    Done(Modification),
    /// Deletes the specified task
    Delete(Modification),
}

#[derive(Args)]
pub struct Modification {
    pub description: Vec<String>,
    #[arg(short, long, value_enum)]
    pub status: Option<Status>,
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
            Some(Commands::Delete(modification)) => handler.delete(&self.filters, modification)?,
            None => handler.next(&self.filters)?,
        }
        Ok(())
    }
}
