use crate::table::{Duration, base_table::TableRow};
use dawn::domain::task::{Description, Index, Task};
use tabled::Tabled;

#[derive(Tabled, Debug)]
#[tabled(rename_all = "PascalCase")]
pub struct NextRow {
    #[tabled(rename = "ID")]
    id: Index,
    age: Duration,
    description: Description,
}

impl TableRow for NextRow {
    fn new(task: Task, now: i64) -> anyhow::Result<Self> {
        Ok(Self {
            id: task.index.ok_or(NextRowError::MissingIndex)?,
            age: Duration::new(&task.entry, now)?,
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
    use crate::{table::duration::DurationError, test_helper::task};
    use tabled::Tabled;

    #[test]
    fn new_succeeds_with_valid_task() {
        let now = 1_000_000;
        let row = NextRow::new(
            task(Some(Index::new(1).unwrap()), "buy milk", now - 30),
            now,
        )
        .unwrap();
        let fields: Vec<String> = row.fields().iter().map(|c| c.to_string()).collect();
        assert_eq!(fields, vec!["1", "30s", "buy milk"]);
    }

    #[test]
    fn new_returns_missing_index_when_index_is_none() {
        let now = 1_000_000;
        let err = NextRow::new(task(None, "buy milk", now), now).unwrap_err();
        assert!(matches!(
            err.downcast_ref::<NextRowError>(),
            Some(NextRowError::MissingIndex)
        ));
    }

    #[test]
    fn new_propagates_duration_error_when_entry_in_future() {
        let now = 1_000_000;
        let err = NextRow::new(
            task(Some(Index::new(1).unwrap()), "buy milk", now + 10),
            now,
        )
        .unwrap_err();
        assert_eq!(
            err.downcast_ref::<DurationError>(),
            Some(&DurationError::InvalidDelta {
                from: now + 10,
                to: now,
            })
        );
    }
}
