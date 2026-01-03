#[derive(Debug, PartialEq)]
use dawn::domain::task::Task;

pub enum Status {
    Pending,
    Completed,
    Deleted,
}

impl Status {
    pub fn get_status(task: &Task) -> Self {
        match (&task.deleted_at, &task.completed_at) {
            (Some(_), _) => Status::Deleted,
            (None, Some(_)) => Status::Completed,
            (None, None) => Status::Pending,
        }
    }
}
