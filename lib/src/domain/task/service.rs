use crate::domain::task::{
    Filter, Task, TaskCreation, TaskModification,
    port::{TaskRepository, TaskService},
};
use uuid::Uuid;

// Generic type 'R' should implement 'TaskRepository' trait
pub struct Service<R: TaskRepository> {
    repo: R,
}

impl<R: TaskRepository> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> TaskService for Service<R>
where
    R: TaskRepository,
{
    fn add(&self, req: &TaskCreation) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        self.repo.create_task(&id, req)
    }

    fn count_pending(&self) -> anyhow::Result<usize> {
        self.repo.count_pending()
    }

    fn list(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        self.repo.list_tasks(filter)
    }

    fn modify(&self, modification: &TaskModification, targets: &[Uuid]) -> anyhow::Result<()> {
        self.repo.update_tasks(modification, targets)
    }

    fn purge(&self, targets: &[Uuid]) -> anyhow::Result<()> {
        self.repo.delete_tasks(targets)
    }
}
