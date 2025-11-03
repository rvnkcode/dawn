use crate::domain::task::port::{TaskRepository, TaskService};

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

impl<R> TaskService for Service<R> where R: TaskRepository {}
