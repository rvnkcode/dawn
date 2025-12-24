use crate::domain::{
    Filter,
    task::{Task, TaskCreation, UniqueID},
};

pub trait TaskService {
    fn add(&self, req: TaskCreation) -> anyhow::Result<()>;
    fn count_pending(&self) -> usize;
    fn next(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn all(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
}

pub trait TaskRepository {
    fn create_task(&self, id: UniqueID, req: TaskCreation) -> anyhow::Result<()>;
    fn count_pending_tasks(&self) -> usize;
    fn get_pending_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
    fn get_all_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>>;
}
