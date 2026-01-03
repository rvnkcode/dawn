use clap::ValueEnum;
use dawn::domain::task::Task;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum Status {
    Pending,
    Completed,
    Deleted,
}

impl Status {
    pub fn get_status(task: &Task) -> Self {
        match (&task.deleted_at, &task.completed_at) {
            (Some(_), _) => Status::Deleted,
            (None, Some(_)) => Status::Completed,
            (None, None) => Status::Pending,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match self {
            Status::Pending => "pending",
            Status::Completed => "completed",
            Status::Deleted => "deleted",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::make_task;

    #[test]
    fn get_status_returns_pending_for_pending_task() {
        let task = make_task("Test", Some(1), false, false);
        assert_eq!(Status::get_status(&task), Status::Pending);
    }

    #[test]
    fn get_status_returns_completed_for_completed_task() {
        let task = make_task("Test", Some(1), true, false);
        assert_eq!(Status::get_status(&task), Status::Completed);
    }

    #[test]
    fn get_status_returns_deleted_for_deleted_task() {
        let task = make_task("Test", Some(1), false, true);
        assert_eq!(Status::get_status(&task), Status::Deleted);
    }

    #[test]
    fn get_status_deleted_takes_priority_over_completed() {
        let task = make_task("Test", Some(1), true, true);
        assert_eq!(Status::get_status(&task), Status::Deleted);
    }

    #[test]
    fn display_pending() {
        assert_eq!(Status::Pending.to_string(), "pending");
    }

    #[test]
    fn display_completed() {
        assert_eq!(Status::Completed.to_string(), "completed");
    }

    #[test]
    fn display_deleted() {
        assert_eq!(Status::Deleted.to_string(), "deleted");
    }
}
