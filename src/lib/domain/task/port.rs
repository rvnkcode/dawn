use crate::domain::task::{TaskCreation, UniqueID};

pub trait TaskService {
    fn add(&self, req: &TaskCreation) -> anyhow::Result<()>;
}

pub trait TaskRepository {
    fn create_task(&self, id: &UniqueID, req: &TaskCreation) -> anyhow::Result<()>;
}
