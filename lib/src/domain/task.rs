pub mod description;
pub use description::Description;
pub mod filter;
pub use filter::Filter;
pub mod index;
pub use index::Index;
pub mod port;
pub mod service;
pub mod timestamp;
pub use timestamp::Timestamp;
pub mod unique_id;
pub use unique_id::UniqueID;

pub struct TaskCreation {
    pub description: Description,
}

pub struct TaskModification {
    pub description: Option<Description>,
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
    pub uid: UniqueID,
    pub index: Option<Index>,
    pub description: Description,
    pub entry: Timestamp,
    pub completed: Option<Timestamp>,
    pub deleted: Option<Timestamp>,
}

#[derive(Eq, PartialEq, Hash)]
pub enum Status {
    Pending,
    Completed,
    Deleted,
    // TODO: Cancelled
}

#[cfg(test)]
mod tests {
    use super::*;

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
