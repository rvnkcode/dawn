use dawn::{domain::task::service::Service, outbound::SQLite};
use dawn_cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::new();
    let mut db = SQLite::new()?;
    db.initialize()?;
    let task_service = Service::new(db);
    cli.handle_command(task_service)?;
    Ok(())
}
