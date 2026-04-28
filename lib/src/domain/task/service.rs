use crate::domain::task::{
    Filter, Task, TaskCreation, TaskModification, UniqueID,
    port::{TaskRepository, TaskService},
};

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
        let id = UniqueID::new();
        self.repo.create_task(&id, req)
    }

    fn count_pending(&self) -> anyhow::Result<usize> {
        self.repo.count_pending()
    }

    fn list(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        self.repo.list_tasks(filter)
    }

    fn modify(&self, modification: &TaskModification, targets: &[&UniqueID]) -> anyhow::Result<()> {
        self.repo.update_tasks(modification, targets)
    }

    fn purge(&self, targets: &[UniqueID]) -> anyhow::Result<()> {
        self.repo.delete_tasks(targets)
    }
}
