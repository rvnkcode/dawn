use crate::domain::{
    Filter,
    task::{
        Task, TaskCreation, TaskModification, UniqueID,
        port::{TaskRepository, TaskService},
    },
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
    fn add(&self, req: TaskCreation) -> anyhow::Result<()> {
        let id = UniqueID::new();
        self.repo.create_task(id, req)
    }

    fn count_pending(&self) -> usize {
        self.repo.count_pending_tasks()
    }

    fn next(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        self.repo.get_pending_tasks(filter)
    }

    fn all(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        self.repo.get_all_tasks(filter)
    }

    fn modify(&self, modification: TaskModification, targets: &[&UniqueID]) -> anyhow::Result<()> {
        self.repo.update_tasks(modification, targets)
    }
}
