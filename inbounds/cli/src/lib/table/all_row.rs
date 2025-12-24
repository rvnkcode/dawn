use crate::table::{Age, Status, TableRow};
use dawn::domain::task::{Description, Index, Task, UniqueID};
use tabled::Tabled;

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct AllRow {
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

impl TableRow for AllRow {
    fn new(task: Task, now: &i64) -> anyhow::Result<Self> {
        let age = Age::new(&task.created_at, now)?;
        let done = match &task.completed_at {
            Some(completed_at) => Some(Age::new(completed_at, now)?),
            None => None,
        };
        let status = match (&task.deleted_at, &task.completed_at) {
            (Some(_), _) => Status::Deleted,
            (None, Some(_)) => Status::Completed,
            (None, None) => Status::Pending,
        };
        Ok(Self {
            id: task.index,
            status,
            uid: task.uid,
            age,
            done,
            description: task.description,
        })
    }
}

impl AllRow {
    #[cfg(test)]
    pub fn status(&self) -> &Status {
        &self.status
    }
}

fn display_done(val: &Option<Age>) -> String {
    match val {
        Some(age) => age.to_string(),
        None => String::new(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(deleted_at: Option<i64>, completed_at: Option<i64>) -> Task {
        Task {
            uid: UniqueID::new(),
            index: None,
            description: Description::new("test task").unwrap(),
            created_at: 1000,
            completed_at,
            deleted_at,
        }
    }

    #[test]
    fn test_status_pending() {
        let task = create_test_task(None, None);
        let now = 2000;
        let row = AllRow::new(task, &now).unwrap();
        assert_eq!(row.status(), &Status::Pending);
    }

    #[test]
    fn test_status_completed() {
        let task = create_test_task(None, Some(1500));
        let now = 2000;
        let row = AllRow::new(task, &now).unwrap();
        assert_eq!(row.status(), &Status::Completed);
    }

    #[test]
    fn test_status_deleted() {
        let task = create_test_task(Some(1500), None);
        let now = 2000;
        let row = AllRow::new(task, &now).unwrap();
        assert_eq!(row.status(), &Status::Deleted);
    }

    #[test]
    fn test_status_deleted_takes_priority() {
        let task = create_test_task(Some(1500), Some(1400));
        let now = 2000;
        let row = AllRow::new(task, &now).unwrap();
        assert_eq!(row.status(), &Status::Deleted);
    }
}
