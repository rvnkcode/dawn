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
    use dawn::domain::task::{Description, Index, UniqueID};

    fn make_task(completed_at: Option<i64>, deleted_at: Option<i64>) -> Task {
        Task {
            uid: "abc12345678".parse::<UniqueID>().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("test").unwrap(),
            created_at: 0,
            completed_at,
            deleted_at,
        }
    }

    #[test]
    fn get_status_returns_pending_for_pending_task() {
        let task = make_task(None, None);
        assert_eq!(Status::get_status(&task), Status::Pending);
    }

    #[test]
    fn get_status_returns_completed_for_completed_task() {
        let task = make_task(Some(1000), None);
        assert_eq!(Status::get_status(&task), Status::Completed);
    }

    #[test]
    fn get_status_returns_deleted_for_deleted_task() {
        let task = make_task(None, Some(1000));
        assert_eq!(Status::get_status(&task), Status::Deleted);
    }

    #[test]
    fn get_status_deleted_takes_priority_over_completed() {
        let task = make_task(Some(1000), Some(2000));
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
