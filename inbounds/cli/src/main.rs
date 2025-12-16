use colored::Colorize;
use dawn::{domain::task, outbound::SQLite};
use dawn_cli::Cli;

fn main() -> anyhow::Result<()> {
    let db = SQLite::new()?;
    let task_service = task::Service::new(db);
    let cli = Cli::new();
    if let Err(e) = cli.handle_command(task_service) {
        eprintln!("{}", e.to_string().white().on_red());
        std::process::exit(1);
    }
    Ok(())
}
