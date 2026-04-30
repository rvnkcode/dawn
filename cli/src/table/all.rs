use crate::table::{Age, base::TableRow};
use dawn::domain::task::{Description, Index, Status, Task, UniqueID};
use tabled::Tabled;

#[derive(Tabled, Debug)]
#[tabled(rename_all = "PascalCase")]
pub(crate) struct AllRow {
    #[tabled(rename = "ID", display("display_index"))]
    id: Option<Index>,
    #[tabled(rename = "St", display("display_status"))]
    status: Status,
    #[tabled(rename = "UID")]
    uid: UniqueID,
    age: Age,
    #[tabled(display("display_done"))]
    done: Option<Age>,
    description: Description,
}

fn display_index(val: &Option<Index>) -> String {
    match val {
        Some(index) => index.to_string(),
        None => String::from("-"),
    }
}

fn display_status(val: &Status) -> String {
    match val {
        Status::Pending => "P".to_string(),
        Status::Completed => "C".to_string(),
        Status::Deleted => "D".to_string(),
    }
}

fn display_done(val: &Option<Age>) -> String {
    match val {
        Some(age) => age.to_string(),
        None => String::new(),
    }
}

impl TableRow for AllRow {
    fn new(task: Task, now: i64) -> anyhow::Result<Self> {
        let status = task.status();
        Ok(Self {
            id: task.index,
            status,
            uid: task.uid,
            age: Age::new(&task.entry, now)?,
            done: task
                .completed
                .map(|done| Age::new(&done, now))
                .transpose()?,
            description: task.description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::task;
    use dawn::domain::task::Timestamp;
    use tabled::Tabled;

    fn fields(row: &AllRow) -> Vec<String> {
        row.fields().iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn new_renders_pending_task_with_index() {
        let now = 1_000_000;
        let task = task(Some(Index::new(1).unwrap()), "buy milk", now - 30);
        let uid = task.uid.to_string();

        let row = AllRow::new(task, now).unwrap();

        assert_eq!(
            fields(&row),
            vec![
                "1".to_string(),
                "P".to_string(),
                uid,
                "30s".to_string(),
                String::new(),
                "buy milk".to_string(),
            ],
        );
    }

    #[test]
    fn new_renders_dash_when_index_is_none() {
        let now = 1_000_000;
        let row = AllRow::new(task(None, "buy milk", now), now).unwrap();
        assert_eq!(fields(&row)[0], "-");
    }

    #[test]
    fn new_renders_completed_status_and_done_age() {
        let now = 1_000_000;
        let mut t = task(Some(Index::new(2).unwrap()), "ship feature", now - 60);
        t.completed = Some(Timestamp::new(now - 10).unwrap());

        let row = AllRow::new(t, now).unwrap();
        let f = fields(&row);

        assert_eq!(f[1], "C");
        assert_eq!(f[4], "10s");
    }

    #[test]
    fn new_renders_deleted_status_with_empty_done() {
        let now = 1_000_000;
        let mut t = task(None, "abandoned", now - 60);
        t.deleted = Some(Timestamp::new(now - 5).unwrap());

        let row = AllRow::new(t, now).unwrap();
        let f = fields(&row);

        assert_eq!(f[1], "D");
        assert_eq!(f[4], "");
    }

    #[test]
    fn new_renders_deleted_status_with_done_when_both_set() {
        let now = 1_000_000;
        let mut t = task(None, "done then deleted", now - 100);
        t.completed = Some(Timestamp::new(now - 50).unwrap());
        t.deleted = Some(Timestamp::new(now - 5).unwrap());

        let row = AllRow::new(t, now).unwrap();
        let f = fields(&row);

        assert_eq!(f[1], "D");
        assert_eq!(f[4], "50s");
    }
}
