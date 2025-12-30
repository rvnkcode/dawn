use std::fmt::{self, Display, Formatter};
use thiserror::Error;

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
