use nanoid::nanoid;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use thiserror::Error;

// task ID
const ID_LENGTH: usize = 11;

#[derive(Debug, PartialEq, Eq)]
pub struct UniqueID(String);

#[derive(Debug, Error)]
#[error("Invalid UniqueID length")]
pub struct UniqueIDLengthError;

impl UniqueID {
    pub fn new() -> Self {
        Self(nanoid!(ID_LENGTH))
    }
}

impl Default for UniqueID {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for UniqueID {
    type Err = UniqueIDLengthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != ID_LENGTH {
            Err(UniqueIDLengthError)
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl Display for UniqueID {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
