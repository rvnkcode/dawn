use clap::{Parser, Subcommand};
use dawn::domain::task::port::TaskService;

use crate::{
    arg::{Creation, Modification},
    error::CliError,
    handler::Handler,
};

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, subcommand_precedence_over_arg = true, version)]
pub struct Cli {
    filter: Vec<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

// TODO: help usage for each command
#[derive(Subcommand)]
enum Command {
    /// Adds a new task
    Add(Creation),
    /// Modifies the existing task with provided arguments
    Modify(Modification),
    /// Marks the specified task as completed
    Done(Modification),
    /// Deletes the specified task
    Delete(Modification),
    /// All tasks
    All(Modification),
    /// Removes the specified tasks from the data files. Causes permanent loss of data
    Purge(Modification),
}

impl Cli {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self, task_service: impl TaskService) -> Result<(), CliError> {
        let handler = Handler::new(task_service);
        match &self.command {
            Some(Command::Add(creation)) => handler.add(&self.filter, &creation.description),
            Some(Command::Modify(modification)) => handler.modify(&self.filter, &modification.mods),
            Some(Command::Done(modification)) => handler.done(&self.filter, &modification.mods),
            Some(Command::Delete(modification)) => handler.delete(&self.filter, &modification.mods),
            Some(Command::All(modification)) => handler.all(&self.filter, &modification.mods),
            Some(Command::Purge(modification)) => handler.purge(&self.filter, &modification.mods),
            None => handler.default(&self.filter),
        }
    }
}
