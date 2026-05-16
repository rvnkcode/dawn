use crate::domain::task::{Description, Timestamp};

#[derive(Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_modification_is_empty_when_all_none() {
        let modification = TaskModification::default();
        assert!(modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_description() {
        let modification = TaskModification {
            description: Some(Description::new("test").unwrap()),
            ..Default::default()
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_completed() {
        let modification = TaskModification {
            completed: Some(Some(Timestamp::new(1700000000).unwrap())),
            ..Default::default()
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_completed_cleared() {
        let modification = TaskModification {
            completed: Some(None),
            ..Default::default()
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_deleted() {
        let modification = TaskModification {
            deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
            ..Default::default()
        };
        assert!(!modification.is_empty());
    }

    #[test]
    fn task_modification_is_not_empty_with_deleted_cleared() {
        let modification = TaskModification {
            deleted: Some(None),
            ..Default::default()
        };
        assert!(!modification.is_empty());
    }
}
