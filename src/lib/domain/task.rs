pub mod description;
pub use description::Description;
pub mod unique_id;
pub use unique_id::UniqueID;

pub struct TaskCreation {
    pub description: Description,
}
