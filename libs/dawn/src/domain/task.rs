pub mod port;
pub mod service;
use nanoid::nanoid;
pub use service::Service;
use thiserror::Error;

// task description
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

// task ID
const ID_LENGTH: usize = 11;

pub struct UniqueId(String);

impl UniqueId {
    pub fn new() -> Self {
        Self(nanoid!(ID_LENGTH))
    }
}
