use crate::domain::task::Index;
use thiserror::Error;

pub struct IndexRange {
    start: Index,
    end: Index,
}

impl IndexRange {
    pub fn new(a: Index, b: Index) -> Result<Self, IndexRangeError> {
        if a == b {
            Err(IndexRangeError)
        } else if a < b {
            Ok(Self { start: a, end: b })
        } else {
            Ok(Self { start: b, end: a })
        }
    }

    pub fn start(&self) -> &Index {
        &self.start
    }

    pub fn end(&self) -> &Index {
        &self.end
    }
}

#[derive(Debug, Error)]
#[error("Start and end indices are the same")]
pub struct IndexRangeError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_ascending_order() {
        let range = IndexRange::new(Index::new(1).unwrap(), Index::new(5).unwrap()).unwrap();

        assert_eq!(range.start(), &Index::new(1).unwrap());
        assert_eq!(range.end(), &Index::new(5).unwrap());
    }

    #[test]
    fn new_with_descending_order_normalizes() {
        let range = IndexRange::new(Index::new(5).unwrap(), Index::new(1).unwrap()).unwrap();

        assert_eq!(range.start(), &Index::new(1).unwrap());
        assert_eq!(range.end(), &Index::new(5).unwrap());
    }

    #[test]
    fn new_with_same_indices_returns_error() {
        let a = Index::new(3).unwrap();
        let b = Index::new(3).unwrap();
        let result = IndexRange::new(a, b);

        assert!(result.is_err());
    }
}
