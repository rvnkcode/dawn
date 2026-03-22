use nanoid::nanoid;
use std::fmt::{self, Display, Formatter};

// 1 ID per second for 309 years = 9B IDs
const ID_LENGTH: usize = 12;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_id_new() {
        let id = UniqueID::new();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), ID_LENGTH);
    }
}
