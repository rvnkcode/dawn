pub mod description;
pub mod filter;
pub mod index;
pub mod index_range;
pub mod modification;
pub mod port;
pub mod service;
pub mod sort;
pub mod status;
pub mod timestamp;
pub mod uuid_prefix;

pub use description::Description;
pub use filter::Filter;
pub use index::Index;
pub use index_range::IndexRange;
pub use modification::TaskModification;
pub use sort::{Direction, SortKey};
pub use status::Status;
pub use timestamp::Timestamp;
use uuid::Uuid;
pub use uuid_prefix::UuidPrefix;

pub struct TaskCreation {
    pub description: Description,
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
}
