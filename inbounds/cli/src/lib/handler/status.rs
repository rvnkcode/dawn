use clap::ValueEnum;
use dawn::domain::task::Task;
use std::fmt::{self, Display, Formatter};

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
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match self {
            Status::Pending => "pending",
            Status::Completed => "completed",
            Status::Deleted => "deleted",
        };
        write!(f, "{}", s)
    }
}
