mod arg;
mod handler;
use handler::Handler;

use crate::{domain::task::port::TaskService, inbound::cli::arg::Creation};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "A command line todo manager.", long_about = None, subcommand_precedence_over_arg = true, version)]
pub struct Cli {
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

    pub fn handle_command(self, task_service: impl TaskService) -> anyhow::Result<()> {
        let handler = Handler::new(task_service);
        match self.command {
            Some(Command::Add(creation)) => handler.add(creation),
            None => Ok(()),
        }
    }
}
