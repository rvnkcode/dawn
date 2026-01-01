use crate::table::{Age, TableRow};
use dawn::domain::task::{Description, Index, Task};
use tabled::Tabled;

#[derive(Debug, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct NextRow {
    #[tabled(rename = "ID")]
    id: Index,
    age: Age,
    description: Description,
}

impl TableRow for NextRow {
    fn new(task: Task, now: &i64) -> anyhow::Result<Self> {
        let age = Age::new(&task.created_at, now)?;
        Ok(Self {
            id: task.index.ok_or(NextRowError::MissingIndex)?,
            age,
            description: task.description,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NextRowError {
    #[error("Index is None")]
    MissingIndex,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dawn::domain::task::UniqueID;

    #[test]
    fn test_missing_index_error() {
        let task = Task {
            uid: UniqueID::default(),
            index: None,
            description: Description::new("test task").unwrap(),
            created_at: 1000,
            completed_at: None,
            deleted_at: None,
        };
        let now = 2000;
        let result = NextRow::new(task, &now);
        let err = result.unwrap_err();
        assert!(err.downcast_ref::<NextRowError>().is_some());
    }
}
