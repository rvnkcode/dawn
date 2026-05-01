use std::{
    fmt::{self, Display, Formatter},
    num::IntErrorKind,
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Index(usize);

#[derive(Debug, Error, PartialEq)]
pub enum IndexError {
    #[error("index must be >= 1, got {0}")]
    TooSmall(usize),
    #[error("index too large: '{0}'")]
    TooLarge(String),
    #[error("invalid index: '{0}'")]
    InvalidFormat(String),
}

impl Index {
    pub fn new(raw: usize) -> Result<Self, IndexError> {
        if raw < 1 {
            Err(IndexError::TooSmall(raw))
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

impl FromStr for Index {
    type Err = IndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let raw: usize = trimmed
            .parse()
            .map_err(|e: std::num::ParseIntError| match e.kind() {
                IntErrorKind::PosOverflow => IndexError::TooLarge(trimmed.to_string()),
                _ => IndexError::InvalidFormat(trimmed.to_string()),
            })?;
        Self::new(raw)
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
        assert_eq!(result, Err(IndexError::TooSmall(0)));
    }

    // FromStr

    #[test]
    fn from_str_valid() {
        let index: Index = "42".parse().unwrap();
        assert_eq!(index.to_string(), "42");
    }

    #[test]
    fn from_str_zero() {
        let result = "0".parse::<Index>();
        assert_eq!(result, Err(IndexError::TooSmall(0)));
    }

    #[test]
    fn from_str_non_numeric() {
        let result = "abc".parse::<Index>();
        assert_eq!(result, Err(IndexError::InvalidFormat("abc".to_string())));
    }

    #[test]
    fn from_str_empty() {
        let result = "".parse::<Index>();
        assert_eq!(result, Err(IndexError::InvalidFormat(String::new())));
    }

    #[test]
    fn from_str_whitespace() {
        let result = "  5  ".parse::<Index>();
        assert_eq!(result.unwrap().to_string(), "5");
    }

    #[test]
    fn from_str_overflow() {
        let raw = "99999999999999999999999999999";
        let result = raw.parse::<Index>();
        assert_eq!(result, Err(IndexError::TooLarge(raw.to_string())));
    }
}
