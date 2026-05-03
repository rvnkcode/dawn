use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    sync::LazyLock,
};

use regex::Regex;
use thiserror::Error;
use uuid::Uuid;

// Taskwarrior parity: 8-char minimum, hyphens at fixed positions, up to full 36 chars.
// Prefix matches via SQL LIKE; see Lexer::isUUID in TW source.
// Exposed without anchors so adapters can embed it in larger regex patterns.
pub const UUID_PREFIX_PATTERN: &str =
    r"[0-9a-f]{8}(?:-[0-9a-f]{0,4}(?:-[0-9a-f]{0,4}(?:-[0-9a-f]{0,4}(?:-[0-9a-f]{0,12})?)?)?)?";

static UUID_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{UUID_PREFIX_PATTERN}$")).unwrap());

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UuidPrefix(String);

#[derive(Debug, Error, PartialEq)]
#[error("invalid UUID prefix: '{0}'")]
pub struct UuidPrefixError(String);

impl UuidPrefix {
    pub fn parse(raw: &str) -> Result<Self, UuidPrefixError> {
        if UUID_PREFIX_RE.is_match(raw) {
            Ok(Self(raw.to_string()))
        } else {
            Err(UuidPrefixError(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for UuidPrefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UuidPrefix {
    type Err = UuidPrefixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// A full UUID (lowercase hyphenated) is always a valid prefix.
impl From<Uuid> for UuidPrefix {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_uuid() {
        let prefix = UuidPrefix::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(prefix.as_str(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn parse_eight_char_minimum() {
        let prefix = UuidPrefix::parse("550e8400").unwrap();
        assert_eq!(prefix.as_str(), "550e8400");
    }

    #[test]
    fn parse_partial_with_trailing_hyphen() {
        let prefix = UuidPrefix::parse("550e8400-").unwrap();
        assert_eq!(prefix.as_str(), "550e8400-");
    }

    #[test]
    fn parse_partial_two_groups() {
        let prefix = UuidPrefix::parse("550e8400-e29b").unwrap();
        assert_eq!(prefix.as_str(), "550e8400-e29b");
    }

    #[test]
    fn parse_rejects_seven_char_prefix() {
        let err = UuidPrefix::parse("550e840").unwrap_err();
        assert_eq!(err, UuidPrefixError("550e840".to_string()));
    }

    #[test]
    fn parse_rejects_oversized_group() {
        // Second group overruns 4-char limit
        assert!(UuidPrefix::parse("550e8400-e29b12").is_err());
    }

    #[test]
    fn parse_rejects_uppercase_hex() {
        assert!(UuidPrefix::parse("550E8400").is_err());
    }

    #[test]
    fn parse_rejects_non_hex_chars() {
        assert!(UuidPrefix::parse("zzzzzzzz").is_err());
    }

    #[test]
    fn parse_rejects_empty() {
        assert!(UuidPrefix::parse("").is_err());
    }

    #[test]
    fn parse_rejects_wildcard_chars() {
        assert!(UuidPrefix::parse("%").is_err());
        assert!(UuidPrefix::parse("_").is_err());
    }

    #[test]
    fn parse_rejects_leading_hyphen() {
        assert!(UuidPrefix::parse("-550e8400").is_err());
    }

    #[test]
    fn from_uuid_yields_full_36_char_prefix() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000"
            .parse::<Uuid>()
            .unwrap();
        let prefix = UuidPrefix::from(uuid);
        assert_eq!(prefix.as_str(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn from_str_delegates_to_parse() {
        let prefix: UuidPrefix = "550e8400".parse().unwrap();
        assert_eq!(prefix.as_str(), "550e8400");
    }

    #[test]
    fn display_round_trips_input() {
        let prefix = UuidPrefix::parse("550e8400-e29b").unwrap();
        assert_eq!(prefix.to_string(), "550e8400-e29b");
    }
}
