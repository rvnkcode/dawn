use clap::ValueEnum;
use dawn::domain::task::Task;

#[derive(Clone, Debug, PartialEq, ValueEnum)]
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

    pub fn to_string(&self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Completed => "completed",
            Status::Deleted => "deleted",
        }
    }
}
