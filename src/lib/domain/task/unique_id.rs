use nanoid::nanoid;

// 1 ID per second for 309 years = 9B IDs
const ID_LENGTH: usize = 12;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_id_new() {
        let id = UniqueID::default();
        assert_eq!(id.0.len(), ID_LENGTH);
    }
}
