use dawn::{domain::task::service::Service, outbound::SQLite};
use dawn_cli::{Cli, CliError};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::new();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            e.write_stderr();
            e.exit_code()
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    // Route to CLI runtime error: exit code 1
    let mut db = SQLite::new().map_err(anyhow::Error::from)?;
    db.initialize().map_err(anyhow::Error::from)?;
    let task_service = Service::new(db);
    cli.run(task_service)
}
