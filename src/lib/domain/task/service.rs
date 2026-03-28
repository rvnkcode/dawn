use crate::domain::task::{
    Filter, Status, Task, TaskCreation, UniqueID,
    port::{TaskRepository, TaskService},
};
use std::collections::HashSet;

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
        self.repo.list_tasks(&Filter {
            statuses: HashSet::from([Status::Pending]),
        })
    }
}
