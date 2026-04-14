use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Index(usize);

#[derive(Debug, Error)]
#[error("index must be >= 1, got {0}")]
pub struct IndexError(usize);

impl Index {
    pub fn new(raw: usize) -> Result<Self, IndexError> {
        if raw < 1 {
            Err(IndexError(raw))
        } else {
            Ok(Self(raw))
        }
    }

    // For SQL query parameters
    pub(crate) fn get(&self) -> usize {
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
    fn index_new_valid() {
        let index = Index::new(1).unwrap();
        assert_eq!(index.to_string(), "1");
    }

    #[test]
    fn index_new_zero() {
        let result = Index::new(0);
        assert!(result.is_err());
    }
}
