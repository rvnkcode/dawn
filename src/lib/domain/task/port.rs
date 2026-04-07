use crate::domain::task::{Filter, Task, TaskCreation, TaskModification, UniqueID};

pub trait TaskService {
    fn add(&self, req: &TaskCreation) -> anyhow::Result<()>;
    fn next(&self) -> anyhow::Result<Vec<Task>>;
    fn modify(&self, modification: TaskModification, targets: &[&UniqueID]) -> anyhow::Result<()>;
}

pub trait TaskRepository {
    fn create_task(&self, id: &UniqueID, req: &TaskCreation) -> anyhow::Result<()>;
    fn list_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn update_tasks(
        &self,
        modification: TaskModification,
        targets: &[&UniqueID],
    ) -> anyhow::Result<()>;
}
