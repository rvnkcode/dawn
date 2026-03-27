pub mod description;
pub use description::Description;
pub mod index;
pub use index::Index;
pub mod port;
pub mod service;
pub mod timestamp;
pub use timestamp::Timestamp;
pub mod unique_id;
pub use unique_id::UniqueID;

pub struct TaskCreation {
    pub description: Description,
}

pub struct Task {
    pub uid: UniqueID,
    pub index: Option<Index>,
    pub description: Description,
    pub entry: Timestamp,
    pub completed: Option<Timestamp>,
    pub deleted: Option<Timestamp>,
}
