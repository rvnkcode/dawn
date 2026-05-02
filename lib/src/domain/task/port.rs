use crate::domain::task::{Filter, Task, TaskCreation, TaskModification, UniqueID};

pub trait TaskService {
    fn add(&self, req: &TaskCreation) -> anyhow::Result<()>;
    fn count_pending(&self) -> anyhow::Result<usize>;
    fn list(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn modify(&self, modification: &TaskModification, targets: &[&UniqueID]) -> anyhow::Result<()>;
    fn purge(&self, targets: &[&UniqueID]) -> anyhow::Result<()>;
}

pub trait TaskRepository {
    fn create_task(&self, id: &UniqueID, req: &TaskCreation) -> anyhow::Result<()>;
    fn count_pending(&self) -> anyhow::Result<usize>;
    fn list_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn update_tasks(
        &self,
        modification: &TaskModification,
        targets: &[&UniqueID],
    ) -> anyhow::Result<()>;
    fn delete_tasks(&self, targets: &[&UniqueID]) -> anyhow::Result<()>;
}
