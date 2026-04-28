use clap::Args;

#[derive(Args)]
pub(crate) struct Creation {
    pub(crate) description: Vec<String>,
}

#[derive(Args)]
pub(crate) struct Modification {
    #[arg(allow_hyphen_values = true)]
    pub(crate) mods: Vec<String>,
}
