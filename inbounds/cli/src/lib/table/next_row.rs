use dawn::domain::task::{Description, Index, Task};
use tabled::Tabled;

use crate::table::Age;

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct NextRow {
    #[tabled(rename = "ID")]
    id: Index,
    age: Age,
    description: Description,
}

impl NextRow {
    pub fn new(task: Task, now: &i64) -> anyhow::Result<Self> {
        let age = Age::new(&task.created_at, now)?;
        Ok(Self {
            id: task.index,
            age,
            description: task.description,
        })
    }
}
