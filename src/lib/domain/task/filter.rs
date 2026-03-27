use crate::domain::task::Status;
use std::collections::HashSet;

pub struct Filter {
    pub statuses: HashSet<Status>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
    }
}
