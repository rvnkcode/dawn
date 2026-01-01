use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl FromStr for Index {
    type Err = IndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: usize = s.parse().map_err(|_| IndexError)?;
        Self::new(n)
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

    #[test]
    fn test_index_from_str_valid() {
        let index: Index = "5".parse().unwrap();
        assert_eq!(index.get(), 5);
    }

    #[test]
    fn test_index_from_str_zero() {
        let result = "0".parse::<Index>();
        assert!(result.is_err());
    }

    #[test]
    fn test_index_from_str_invalid() {
        let result = "abc".parse::<Index>();
        assert!(result.is_err());
    }
}
