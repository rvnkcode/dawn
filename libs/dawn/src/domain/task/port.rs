use crate::domain::task::{Description, Task, UniqueID};

pub trait TaskService {
    fn add(&self, description: Description) -> anyhow::Result<()>;
    fn count_pending(&self) -> usize;
    // TODO: parse filters and handling?
    fn next(&self) -> anyhow::Result<Vec<Task>>;
}

pub trait TaskRepository {
    fn create_task(&self, id: UniqueID, description: Description) -> anyhow::Result<()>;
    fn count_pending_tasks(&self) -> usize;
    fn get_pending_tasks(&self) -> anyhow::Result<Vec<Task>>;
}
