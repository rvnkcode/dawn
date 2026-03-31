use crate::domain::task::Status;
use std::collections::HashSet;

#[derive(Default)]
pub struct Filter {
    statuses: HashSet<Status>,
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_statuses(self, statuses: impl IntoIterator<Item = Status>) -> Self {
        Self {
            statuses: statuses.into_iter().collect(),
            ..self
        }
    }

    pub fn statuses(&self) -> &HashSet<Status> {
        &self.statuses
    }

    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
    }
}
