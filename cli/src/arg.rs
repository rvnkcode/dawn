use clap::Args;
use dawn::domain::task::Status;

#[derive(Args)]
pub(crate) struct Creation {
    pub(crate) description: Vec<String>,
}

#[derive(Args)]
pub(crate) struct Modification {
    pub(crate) mods: Vec<String>,
    #[arg(long, value_parser = parse_status)]
    pub(crate) status: Option<Status>,
}

fn parse_status(s: &str) -> Result<Status, String> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(Status::Pending),
        "completed" => Ok(Status::Completed),
        "deleted" => Ok(Status::Deleted),
        _ => Err("expected pending, completed, or deleted".to_string()),
    }
}

#[derive(Args)]
pub(crate) struct ModificationOnly {
    #[arg(hide = true)]
    pub(crate) mods: Vec<String>,
}
