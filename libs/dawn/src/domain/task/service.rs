use crate::domain::task::{
    Task, TaskCreation, UniqueID,
    port::{TaskRepository, TaskService},
};

// Generic type 'R' should implement 'TaskRepository' trait
pub struct Service<R>
where
    R: TaskRepository,
{
    repo: R,
}
impl<R> Service<R>
where
    R: TaskRepository,
{
    pub fn new(repo: R) -> Self {
        Service { repo }
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

    fn next(&self) -> anyhow::Result<Vec<Task>> {
        self.repo.get_pending_tasks()
    }

    fn all(&self) -> anyhow::Result<Vec<Task>> {
        self.repo.get_all_tasks()
    }
}
