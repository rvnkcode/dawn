pub mod description;
pub use description::Description;
pub mod port;
pub mod service;
pub mod unique_id;
pub use service::Service;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;
pub use unique_id::UniqueID;

// task index
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(usize);

#[derive(Debug, Error)]
#[error("Invalid range")]
pub struct IndexError;

impl Index {
    pub fn new(raw: usize) -> Result<Self, IndexError> {
        if raw < 1 {
            Err(IndexError)
        } else {
            Ok(Index(raw))
        }
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq)]
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

pub struct TaskModification {
    pub description: Option<Description>,
}

impl TaskModification {
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_new_valid() {
        let result = Index::new(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_index_new_zero() {
        let result = Index::new(0);
        assert!(result.is_err());
    }
}
