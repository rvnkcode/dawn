use crate::domain::task::{Description, Task, UniqueID};

pub trait TaskService {
    fn add(&self, description: Description) -> anyhow::Result<Task>;
    // TODO: parse filters and handling?
    fn next(&self) -> anyhow::Result<Vec<Task>>;
}

pub trait TaskRepository {
    fn create_task(&self, id: UniqueID, description: Description) -> anyhow::Result<Task>;
    fn count_pending_tasks(&self) -> usize;
    fn get_pending_tasks(&self) -> anyhow::Result<Vec<Task>>;
}
