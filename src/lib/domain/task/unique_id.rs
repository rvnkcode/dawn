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

#[derive(Debug, PartialEq, Error)]
pub enum UniqueIDParseError {
    #[error("UniqueID must be {ID_LENGTH} characters, got {0}")]
    InvalidLength(usize),
    #[error("UniqueID contains invalid character: '{0}'")]
    InvalidCharacter(char),
}

impl FromStr for UniqueID {
    type Err = UniqueIDParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != ID_LENGTH {
            return Err(UniqueIDParseError::InvalidLength(s.len()));
        }
        if let Some(c) = s
            .chars()
            .find(|&c| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        {
            return Err(UniqueIDParseError::InvalidCharacter(c));
        }
        Ok(Self(s.to_string()))
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
    fn from_str_valid() {
        let result = "abcdefghijkl".parse::<UniqueID>(); // 12 chars
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "abcdefghijkl");
    }

    #[test]
    fn from_str_short() {
        let result = "abcdefghijk".parse::<UniqueID>(); // 11 chars
        assert_eq!(result, Err(UniqueIDParseError::InvalidLength(11)));
    }

    #[test]
    fn from_str_long() {
        let result = "abcdefghijklm".parse::<UniqueID>(); // 13 chars
        assert_eq!(result, Err(UniqueIDParseError::InvalidLength(13)));
    }

    #[test]
    fn from_str_with_underscore_and_hyphen() {
        let result = "abc_def-ghij".parse::<UniqueID>();
        assert!(result.is_ok());
    }

    #[test]
    fn from_str_with_space() {
        let result = "abcdef ghijl".parse::<UniqueID>();
        assert_eq!(result, Err(UniqueIDParseError::InvalidCharacter(' ')));
    }

    #[test]
    fn from_str_with_non_ascii() {
        let result = "abcdefghijk!".parse::<UniqueID>();
        assert_eq!(result, Err(UniqueIDParseError::InvalidCharacter('!')));
    }
}
