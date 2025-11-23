use crate::context::AppContext;
use crate::handler::Handler;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
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
}

#[derive(Args)]
pub struct Modification {
    pub description: Vec<String>,
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn handle_command(&self, task_service: impl TaskService) -> anyhow::Result<()> {
        let context = AppContext::new(task_service);
        let handler = Handler::new(context);

        match &self.command {
            Some(Commands::Add(modification)) => match handler.add(&self.filters, modification) {
                Ok(task) => {
                    println!("Created task {}.", task.index);
                }
                Err(e) => {
                    eprintln!("{}", e.to_string().white().on_red());
                    std::process::exit(1);
                }
            },
            None => {
                // TODO: next list
            }
        }
        Ok(())
    }
}
