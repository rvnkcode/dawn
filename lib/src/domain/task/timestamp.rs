use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Timestamp(i64);

#[derive(Debug, Error)]
#[error("Timestamp must be >= 0, got {0}")]
pub struct TimestampError(i64);

impl Timestamp {
    pub fn new(raw: i64) -> Result<Self, TimestampError> {
        if raw < 0 {
            Err(TimestampError(raw))
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) fn get(&self) -> i64 {
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
        let result = Timestamp::new(0);
        assert!(result.is_ok());
    }

    #[test]
    fn timestamp_new_invalid() {
        let result = Timestamp::new(-1);
        assert!(result.is_err());
    }
}
