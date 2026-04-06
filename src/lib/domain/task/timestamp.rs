use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Timestamp(i64);

#[derive(Debug, Error)]
#[error("Invalid range")]
pub struct TimestampError;

impl Timestamp {
    pub fn new(raw: i64) -> Result<Self, TimestampError> {
        if raw < 0 {
            Err(TimestampError)
        } else {
            Ok(Self(raw))
        }
    }

    pub fn get(&self) -> i64 {
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
