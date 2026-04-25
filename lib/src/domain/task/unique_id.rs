use nanoid::nanoid;
use regex::Regex;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    sync::LazyLock,
};
use thiserror::Error;

// 1 ID per second for 309 years = 9B IDs
const ID_LENGTH: usize = 12;

pub const UID_PATTERN: &str = r"[A-Za-z0-9_-]{12}";

static UID_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!("^{UID_PATTERN}$")).unwrap());

#[derive(Debug, Eq, Hash, PartialEq)]
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
#[error("invalid unique ID: '{0}'")]
pub struct UniqueIDParseError(String);

impl FromStr for UniqueID {
    type Err = UniqueIDParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_string();
        if !UID_RE.is_match(&trimmed) {
            return Err(UniqueIDParseError(trimmed));
        }
        Ok(Self(trimmed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_id_new() {
        let id = UniqueID::new();
        assert_eq!(id.to_string().len(), ID_LENGTH);
    }

    #[test]
    fn from_str_valid() {
        let result = "abcdefghi-_0".parse::<UniqueID>(); // 12 chars
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "abcdefghi-_0");
    }

    #[test]
    fn from_str_short() {
        let result = "abcdefghijk".parse::<UniqueID>(); // 11 chars
        assert!(result.is_err());
    }

    #[test]
    fn from_str_long() {
        let result = "abcdefghijklm".parse::<UniqueID>(); // 13 chars
        assert!(result.is_err());
    }

    #[test]
    fn from_str_with_space() {
        let result = "abcdef ghijl".parse::<UniqueID>();
        assert!(result.is_err());
    }

    #[test]
    fn from_str_with_disallowed_character() {
        let result = "abcdefghijk!".parse::<UniqueID>();
        assert!(result.is_err());
    }

    #[test]
    fn from_str_whitespace() {
        let result = "  abcdefghijkl  ".parse::<UniqueID>();
        assert_eq!(result.unwrap().to_string(), "abcdefghijkl");
    }

    #[test]
    fn from_str_whitespace_only() {
        let result = "   ".parse::<UniqueID>();
        assert!(result.is_err());
    }
}
