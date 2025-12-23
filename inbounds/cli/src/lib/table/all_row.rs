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
