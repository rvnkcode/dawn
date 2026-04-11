use crate::domain::task::{Description, TaskCreation};
use clap::Args;

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
