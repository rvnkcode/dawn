use thiserror::Error;

pub struct Index(usize);

#[derive(Debug, Error)]
#[error("Invalid range")]
pub struct IndexError;

impl Index {
    pub fn new(raw: usize) -> Result<Self, IndexError> {
        if raw < 1 {
            Err(IndexError)
        } else {
            Ok(Self(raw))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_new_valid() {
        let result = Index::new(1);
        assert!(result.is_ok());
    }

    #[test]
    fn index_new_zero() {
        let result = Index::new(0);
        assert!(result.is_err());
    }
}
