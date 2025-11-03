use dawn::{domain::task, outbound::SQLite};
use dawn_cli::Cli;

fn main() -> anyhow::Result<()> {
    let db = SQLite::new()?;
    let task_service = task::Service::new(db);
    let cli = Cli::new();
    cli.handle_command(task_service)
}
