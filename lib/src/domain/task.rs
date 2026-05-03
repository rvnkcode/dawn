pub mod description;
pub use description::Description;
pub mod filter;
pub use filter::Filter;
pub mod index;
pub use index::Index;
pub mod index_range;
pub use index_range::IndexRange;
pub mod port;
pub mod service;
pub mod timestamp;
pub use timestamp::Timestamp;
pub mod uuid_prefix;
pub use uuid_prefix::UuidPrefix;

use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

pub struct TaskCreation {
    pub description: Description,
}

pub struct TaskModification {
    pub description: Option<Description>,
    // TODO: entry
    pub completed: Option<Option<Timestamp>>,
    pub deleted: Option<Option<Timestamp>>,
}

impl TaskModification {
    pub fn is_empty(&self) -> bool {
        self.description.is_none() && self.completed.is_none() && self.deleted.is_none()
    }
}

#[derive(Debug, PartialEq)]
pub struct Task {
    pub uuid: Uuid,
    pub index: Option<Index>,
    pub description: Description,
    pub entry: Timestamp,
    pub completed: Option<Timestamp>,
    pub deleted: Option<Timestamp>,
    pub modified: Timestamp,
}

impl Task {
    pub fn status(&self) -> Status {
        if self.deleted.is_some() {
            return Status::Deleted;
        }
        if self.completed.is_some() {
            return Status::Completed;
        }
        Status::Pending
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Pending,
    Completed,
    Deleted,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::Completed => write!(f, "Completed"),
            Status::Deleted => write!(f, "Deleted"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create test task with completed or deleted status
    fn task_with(completed: Option<Timestamp>, deleted: Option<Timestamp>) -> Task {
        Task {
            uuid: Uuid::new_v4(),
            index: None,
            description: Description::new("test").unwrap(),
            entry: Timestamp::new(1700000000).unwrap(),
            completed,
            deleted,
            modified: Timestamp::new(1700000000).unwrap(),
        }
    }

    #[test]
    fn status_is_pending_when_not_completed_or_deleted() {
        let task = task_with(None, None);
        assert_eq!(task.status(), Status::Pending);
        assert_eq!(task.status().to_string(), "Pending");
    }

    #[test]
    fn status_is_completed_when_completed_is_some() {
        let task = task_with(Some(Timestamp::new(1700000001).unwrap()), None);
        assert_eq!(task.status(), Status::Completed);
        assert_eq!(task.status().to_string(), "Completed");
    }

    #[test]
    fn status_is_deleted_when_deleted_is_some() {
        let task = task_with(None, Some(Timestamp::new(1700000001).unwrap()));
        assert_eq!(task.status(), Status::Deleted);
        assert_eq!(task.status().to_string(), "Deleted");
    }

    #[test]
    fn status_is_deleted_when_both_completed_and_deleted() {
        let task = task_with(
            Some(Timestamp::new(1700000001).unwrap()),
            Some(Timestamp::new(1700000002).unwrap()),
        );
        assert_eq!(task.status(), Status::Deleted);
    }

    #[test]
    fn task_modification_is_empty_when_all_none() {
        let modification = TaskModification {
            description: None,
            completed: None,
            deleted: None,
        };
        assert!(modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_description() {
        let modification = TaskModification {
            description: Some(Description::new("test").unwrap()),
            completed: None,
            deleted: None,
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_completed() {
        let modification = TaskModification {
            description: None,
            completed: Some(Some(Timestamp::new(1700000000).unwrap())),
            deleted: None,
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_completed_cleared() {
        let modification = TaskModification {
            description: None,
            completed: Some(None),
            deleted: None,
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_deleted() {
        let modification = TaskModification {
            description: None,
            completed: None,
            deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_deleted_cleared() {
        let modification = TaskModification {
            description: None,
            completed: None,
            deleted: Some(None),
        };
        assert!(!modification.is_empty());
    }
}
