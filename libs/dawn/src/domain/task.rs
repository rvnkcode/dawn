pub mod port;
pub mod service;
use nanoid::nanoid;
pub use service::Service;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

// task description
#[derive(Debug)]
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

#[derive(Debug)]
pub struct UniqueID(String);

#[derive(Debug, Error)]
#[error("Invalid UniqueID length")]
pub struct UniqueIDLengthError;

impl UniqueID {
    pub fn new() -> Self {
        Self(nanoid!(ID_LENGTH))
    }

    pub fn from_str(raw: &str) -> Result<Self, UniqueIDLengthError> {
        if raw.len() != ID_LENGTH {
            Err(UniqueIDLengthError)
        } else {
            Ok(Self(raw.to_string()))
        }
    }
}

impl Display for UniqueID {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// task index
#[derive(Debug)]
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
    pub index: Option<Index>,
    pub description: Description,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

pub struct TaskCreation {
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

    #[test]
    fn test_unique_id_from_str_valid() {
        let result = UniqueID::from_str("abcdefghijk"); // 11 chars
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "abcdefghijk");
    }

    #[test]
    fn test_unique_id_from_str_too_short() {
        let result = UniqueID::from_str("abcdefghij"); // 10 chars
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_id_from_str_too_long() {
        let result = UniqueID::from_str("abcdefghijkl"); // 12 chars
        assert!(result.is_err());
    }
}
