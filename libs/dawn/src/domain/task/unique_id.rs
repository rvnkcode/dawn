use nanoid::nanoid;
use std::fmt::{self, Display, Formatter};

const ID_LENGTH: usize = 11;

pub struct UniqueID(String);

impl UniqueID {
    pub fn new() -> Self {
        Self(nanoid!(ID_LENGTH))
    }
}

impl Default for UniqueID {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for UniqueID {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_id_new() {
        let id = UniqueID::default();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), ID_LENGTH);
    }
}
