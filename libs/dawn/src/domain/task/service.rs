use crate::domain::task::{
    Description, Task, UniqueID,
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
    fn add(&self, description: Description) -> anyhow::Result<()> {
        let id = UniqueID::new();
        self.repo.create_task(id, description)
    }

    fn count_pending(&self) -> usize {
        self.repo.count_pending_tasks()
    }

    fn next(&self) -> anyhow::Result<Vec<Task>> {
        self.repo.get_pending_tasks()
    }
}
