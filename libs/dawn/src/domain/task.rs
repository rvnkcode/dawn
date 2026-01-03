pub mod description;
pub use description::Description;
pub mod index;
pub use index::Index;
pub mod port;
pub mod service;
pub mod unique_id;
pub use service::Service;
pub use unique_id::UniqueID;

#[derive(Debug, PartialEq)]
pub struct Task {
    pub uid: UniqueID,
    pub index: Option<Index>,
    pub description: Description,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

pub struct TaskCreation {
    pub description: Description,
}

#[derive(Debug)]
pub struct TaskModification {
    pub description: Option<Description>,
    pub completed_at: Option<Option<i64>>,
    pub deleted_at: Option<Option<i64>>,
}

impl TaskModification {
    pub fn is_empty(&self) -> bool {
        self.description.is_none() && self.completed_at.is_none() && self.deleted_at.is_none()
    }
}
