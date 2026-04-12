use dawn::{domain::task::service::Service, inbound::Cli, outbound::SQLite};

fn main() -> anyhow::Result<()> {
    let cli = Cli::new();
    let mut db = SQLite::new()?;
    db.initialize()?;
    let task_service = Service::new(db);
    cli.handle_command(task_service)?;
    Ok(())
}
