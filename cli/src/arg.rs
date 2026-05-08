use clap::Args;
use dawn::domain::task::Status;

#[derive(Args)]
pub(crate) struct Creation {
    pub(crate) description: Vec<String>,
}

#[derive(Args)]
pub(crate) struct Modification {
    pub(crate) mods: Vec<String>,
    #[arg(long)]
    pub(crate) status: Option<Status>,
}
