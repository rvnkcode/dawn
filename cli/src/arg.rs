use clap::Args;
use dawn::domain::task::{Description, TaskCreation};

#[derive(Args)]
pub(crate) struct Creation {
    description: Description,
}

impl From<Creation> for TaskCreation {
    fn from(m: Creation) -> Self {
        Self {
            description: m.description,
        }
    }
}
