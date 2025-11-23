pub mod port;
pub mod service;
use nanoid::nanoid;
pub use service::Service;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

// task description
pub struct Description(String);

#[derive(Debug, Error)]
#[error("Additional Text must be provided.")]
pub struct DescriptionEmptyError;

impl Description {
    pub fn new(raw: &str) -> Result<Self, DescriptionEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(DescriptionEmptyError)
        } else {
            Ok(Description(trimmed.to_string()))
        }
    }
}

impl Display for Description {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// task ID
const ID_LENGTH: usize = 11;

pub struct UniqueID(String);

impl UniqueID {
    pub fn new() -> Self {
        Self(nanoid!(ID_LENGTH))
    }
}

impl Display for UniqueID {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// task index
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
}

impl Display for Index {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Task {
    pub uid: UniqueID,
    pub index: Index,
    pub description: Description,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_new_valid() {
        let result = Description::new("Valid description");
        assert!(result.is_ok());
    }

    #[test]
    fn test_description_new_empty_string() {
        let result = Description::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_description_new_whitespace_only() {
        let result = Description::new("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_description_new_trims_whitespace() {
        let result = Description::new("  hello world  ");
        assert!(result.is_ok());
        let desc = result.unwrap();
        assert_eq!(desc.to_string(), "hello world");
    }

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

    #[test]
    fn test_unique_id_new() {
        let id = UniqueID::new();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), ID_LENGTH);
    }
}
