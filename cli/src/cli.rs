use crate::{arg::Creation, error::CliError, filter::ParsedFilters, handler::Handler};
use clap::{Parser, Subcommand};
use dawn::domain::task::port::TaskService;

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, subcommand_precedence_over_arg = true, version)]
pub struct Cli {
    filter: Vec<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Adds a new task
    Add(Creation),
}

impl Cli {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn handle_command(self, task_service: impl TaskService) -> Result<(), CliError> {
        let handler = Handler::new(task_service);
        match self.command {
            Some(Command::Add(creation)) => handler.add(&self.filter, &creation.description),
            None => {
                let filter = ParsedFilters::new(&self.filter).into_set_filter();
                handler.next(filter)
            }
        }
    }
}
