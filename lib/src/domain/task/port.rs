use crate::domain::task::{Filter, Task, TaskCreation, TaskModification};
use uuid::Uuid;

pub trait TaskService {
    fn add(&self, req: &TaskCreation) -> anyhow::Result<()>;
    fn count_pending(&self) -> anyhow::Result<usize>;
    fn list(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn modify(&self, modification: &TaskModification, targets: &[Uuid]) -> anyhow::Result<()>;
    fn purge(&self, targets: &[Uuid]) -> anyhow::Result<()>;
}

pub trait TaskRepository {
    fn create_task(&self, id: &Uuid, req: &TaskCreation) -> anyhow::Result<()>;
    fn count_pending(&self) -> anyhow::Result<usize>;
    fn list_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn update_tasks(&self, modification: &TaskModification, targets: &[Uuid])
    -> anyhow::Result<()>;
    fn delete_tasks(&self, targets: &[Uuid]) -> anyhow::Result<()>;
}
