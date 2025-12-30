use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Description(String);

#[derive(Debug, Error)]
#[error("Additional Text must be provided.")]
pub struct DescriptionEmptyError;

impl Description {
    pub fn new(raw: &str) -> Result<Self, DescriptionEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(DescriptionEmptyError)
        } else {
            Ok(Description(trimmed.to_string()))
        }
    }
}

impl Display for Description {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_new_valid() {
        let result = Description::new("Valid description");
        assert!(result.is_ok());
    }

    #[test]
    fn test_description_new_empty_string() {
        let result = Description::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_description_new_whitespace_only() {
        let result = Description::new("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_description_new_trims_whitespace() {
        let result = Description::new("  hello world  ");
        assert!(result.is_ok());
        let desc = result.unwrap();
        assert_eq!(desc.to_string(), "hello world");
    }
}
