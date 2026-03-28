use crate::domain::task::{Filter, Task, TaskCreation, UniqueID};

pub trait TaskService {
    fn add(&self, req: &TaskCreation) -> anyhow::Result<()>;
    fn next(&self) -> anyhow::Result<Vec<Task>>;
}

pub trait TaskRepository {
    fn create_task(&self, id: &UniqueID, req: &TaskCreation) -> anyhow::Result<()>;
    fn list_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
}
