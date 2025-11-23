use crate::domain::task::{Description, Task, UniqueID};

pub trait TaskService {
    fn add(&self, description: Description) -> anyhow::Result<Task>;
}

pub trait TaskRepository {
    fn create_task(&self, id: UniqueID, description: Description) -> anyhow::Result<Task>;
    fn count_pending_tasks(&self) -> anyhow::Result<usize>;
}
