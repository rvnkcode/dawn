pub mod description;
pub use description::Description;
pub mod filter;
pub use filter::Filter;
pub mod index;
pub use index::Index;
pub mod port;
pub mod service;
pub mod timestamp;
pub use timestamp::Timestamp;
pub mod unique_id;
pub use unique_id::UniqueID;

#[derive(Debug, PartialEq)]
pub struct TaskCreation {
    pub description: Description,
}

#[derive(Debug, PartialEq)]
pub struct Task {
    pub uid: UniqueID,
    pub index: Option<Index>,
    pub description: Description,
    pub entry: Timestamp,
    pub completed: Option<Timestamp>,
    pub deleted: Option<Timestamp>,
}

#[derive(Eq, PartialEq, Hash)]
pub enum Status {
    Pending,
    Completed,
    Deleted,
    // TODO: Cancelled
}
