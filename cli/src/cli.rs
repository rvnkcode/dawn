use crate::{
    arg::{Creation, Modification},
    error::CliError,
    handler::Handler,
};
use clap::{Parser, Subcommand};
use dawn::domain::task::port::TaskService;

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, subcommand_precedence_over_arg = true, version)]
pub struct Cli {
    #[arg(allow_hyphen_values = true)]
    filter: Vec<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Adds a new task
    Add(Creation),
    /// Modifies the existing task with provided arguments
    Modify(Modification),
}

impl Cli {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn handle_command(&self, task_service: impl TaskService) -> Result<(), CliError> {
        let handler = Handler::new(task_service);
        match &self.command {
            Some(Command::Add(creation)) => handler.add(&self.filter, &creation.description),
            Some(Command::Modify(modification)) => handler.modify(&self.filter, &modification.mods),
            None => handler.default(&self.filter),
        }
    }
}
