use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Timestamp(i64);

#[derive(Debug, Error)]
#[error("timestamp must be >= 0, got {0}")]
pub struct TimestampError(i64);

impl Timestamp {
    pub fn new(raw: i64) -> Result<Self, TimestampError> {
        if raw < 0 {
            Err(TimestampError(raw))
        } else {
            Ok(Self(raw))
        }
    }

    pub fn as_seconds(&self) -> i64 {
        self.0
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_new_valid() {
        let result = Timestamp::new(0).unwrap();
        assert_eq!(result.to_string(), "0");
    }

    #[test]
    fn timestamp_new_invalid() {
        let result = Timestamp::new(-1);
        assert!(result.is_err());
    }
}
