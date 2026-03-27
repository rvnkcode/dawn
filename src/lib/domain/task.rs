pub mod description;
pub use description::Description;
pub mod index;
pub use index::Index;
pub mod port;
pub mod service;
pub mod unique_id;
pub use unique_id::UniqueID;

pub struct TaskCreation {
    pub description: Description,
}
