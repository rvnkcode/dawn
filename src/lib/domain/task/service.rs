use crate::domain::task::{
    TaskCreation, UniqueID,
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
        let id = UniqueID::default();
        self.repo.create_task(&id, req)
    }
}
