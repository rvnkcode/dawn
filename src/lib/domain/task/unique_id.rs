use nanoid::nanoid;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use thiserror::Error;

// 1 ID per second for 309 years = 9B IDs
const ID_LENGTH: usize = 12;

#[derive(Debug, PartialEq)]
pub struct UniqueID(String);

impl UniqueID {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(nanoid!(ID_LENGTH))
    }
}

impl Display for UniqueID {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("Invalid UniqueID length")]
pub struct UniqueIDLengthError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_id_new() {
        let id = UniqueID::new();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), ID_LENGTH);
    }

    #[test]
    fn unique_id_from_str_valid() {
        let result = "abcdefghijkl".parse::<UniqueID>(); // 12 chars
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "abcdefghijkl");
    }

    #[test]
    fn unique_id_from_str_short() {
        let result = "abcdefghijk".parse::<UniqueID>(); // 11 chars
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_id_from_str_long() {
        let result = "abcdefghijklm".parse::<UniqueID>(); // 13 chars
        assert!(result.is_err());
    }
}
