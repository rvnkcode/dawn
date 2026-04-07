use crate::domain::task::{
    Filter, Status, Task, TaskCreation, TaskModification, UniqueID,
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

    fn next(&self) -> anyhow::Result<Vec<Task>> {
        self.repo
            .list_tasks(&Filter::new().with_statuses([Status::Pending]))
    }

    fn modify(&self, modification: &TaskModification, targets: &[UniqueID]) -> anyhow::Result<()> {
        self.repo.update_tasks(modification, targets)
    }
}
