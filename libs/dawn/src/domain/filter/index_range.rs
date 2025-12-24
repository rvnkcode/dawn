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
        let a = Index::new(1).unwrap();
        let b = Index::new(5).unwrap();
        let range = IndexRange::new(a, b).unwrap();

        assert_eq!(range.start(), &a);
        assert_eq!(range.end(), &b);
    }

    #[test]
    fn new_with_descending_order_normalizes() {
        let a = Index::new(5).unwrap();
        let b = Index::new(1).unwrap();
        let range = IndexRange::new(a, b).unwrap();

        assert_eq!(range.start(), &b);
        assert_eq!(range.end(), &a);
    }

    #[test]
    fn new_with_same_indices_returns_error() {
        let a = Index::new(3).unwrap();
        let b = Index::new(3).unwrap();
        let result = IndexRange::new(a, b);

        assert!(result.is_err());
    }
}
