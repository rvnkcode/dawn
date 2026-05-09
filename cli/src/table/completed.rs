use chrono::Local;
use dawn::domain::task::{Description, Index, Task};
use tabled::Tabled;

use crate::table::{
    Age,
    base::TableRow,
    date_format::{DATE_FMT, format_absolute},
    display_index, get_prefix,
};

#[derive(Tabled, Debug)]
#[tabled(rename_all = "PascalCase")]
pub(crate) struct CompletedRow {
    #[tabled(rename = "ID", display("display_index"))]
    id: Option<Index>,
    #[tabled(rename = "UUID")]
    uuid: String,
    created: String,
    completed: String,
    age: Age,
    description: Description,
}

impl TableRow for CompletedRow {
    fn new(task: Task, now: i64) -> anyhow::Result<Self> {
        Ok(Self {
            id: task.index,
            uuid: get_prefix(&task.uuid),
            created: format_absolute(&task.entry, &Local, DATE_FMT)?.to_string(),
            completed: format_absolute(
                &task
                    .completed
                    .ok_or_else(|| anyhow::anyhow!("Missing completed timestamp"))?,
                &Local,
                DATE_FMT,
            )?
            .to_string(),
            age: Age::new(&task.entry, now)?,
            description: task.description,
        })
    }
}

#[cfg(test)]
mod tests {
    use dawn::domain::task::Timestamp;
    use tabled::Tabled;

    use super::*;
    use crate::test_helper::task;

    const DAY: i64 = 86_400;

    fn fields(row: &CompletedRow) -> Vec<String> {
        row.fields().iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn new_renders_completed_task() {
        let now = 1_000_000;
        let entry_secs = now - 3 * DAY;
        let completed_secs = now - 60;
        let mut t = task(Some(Index::new(1).unwrap()), "ship feature", entry_secs);
        t.completed = Some(Timestamp::new(completed_secs).unwrap());
        let uuid = t.uuid;

        let row = CompletedRow::new(t, now).unwrap();
        let f = fields(&row);

        let expected_created =
            format_absolute(&Timestamp::new(entry_secs).unwrap(), &Local, DATE_FMT)
                .unwrap()
                .to_string();
        let expected_completed =
            format_absolute(&Timestamp::new(completed_secs).unwrap(), &Local, DATE_FMT)
                .unwrap()
                .to_string();

        assert_eq!(f[0], "1");
        assert_eq!(f[1].len(), 8);
        assert!(f[1].chars().all(|c| c.is_ascii_hexdigit()));
        assert!(uuid.to_string().starts_with(&f[1]));
        assert_eq!(f[2], expected_created);
        assert_eq!(f[3], expected_completed);
        assert_ne!(f[2], f[3]); // 3-day gap → distinct dates in any local timezone
        assert_eq!(f[4], "3d");
        assert_eq!(f[5], "ship feature");
    }

    #[test]
    fn new_renders_dash_when_index_is_none() {
        let now = 1_000_000;
        let mut t = task(None, "buy milk", now - 60);
        t.completed = Some(Timestamp::new(now - 10).unwrap());

        let row = CompletedRow::new(t, now).unwrap();
        assert_eq!(fields(&row)[0], "-");
    }

    #[test]
    fn new_returns_error_when_completed_is_none() {
        let now = 1_000_000;
        let err = CompletedRow::new(task(Some(Index::new(1).unwrap()), "x", now), now).unwrap_err();
        assert!(err.to_string().contains("Missing completed timestamp"));
    }

    #[test]
    fn new_propagates_error_when_completed_is_out_of_range() {
        let now = 1_000_000;
        let mut t = task(Some(Index::new(1).unwrap()), "x", now);
        t.completed = Some(Timestamp::new(i64::MAX).unwrap());

        let err = CompletedRow::new(t, now).unwrap_err();
        assert!(err.to_string().contains("out of DateTime range"));
    }
}
