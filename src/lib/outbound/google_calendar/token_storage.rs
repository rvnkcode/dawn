use std::hash::{DefaultHasher, Hash, Hasher};

use async_trait::async_trait;
use keyring::Entry;
use yup_oauth2::storage::{TokenInfo, TokenStorage, TokenStorageError};

const SERVICE_NAME: &str = "dawn";

pub struct KeyringTokenStorage;

impl KeyringTokenStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TokenStorage for KeyringTokenStorage {
    async fn set(&self, scopes: &[&str], token: TokenInfo) -> Result<(), TokenStorageError> {
        let key = scopes_into_key(scopes);
        let entry = Entry::new(SERVICE_NAME, &key)
            .map_err(|e| TokenStorageError::Other(e.to_string().into()))?;
        let serialized_token = serde_json::to_string(&token)
            .map_err(|e| TokenStorageError::Other(e.to_string().into()))?;
        entry
            .set_password(&serialized_token)
            .map_err(|e| TokenStorageError::Other(e.to_string().into()))?;
        Ok(())
    }

    async fn get(&self, scopes: &[&str]) -> Option<TokenInfo> {
        let key = scopes_into_key(scopes);
        let entry = Entry::new(SERVICE_NAME, &key).ok()?;
        let json = entry.get_password().ok()?;
        serde_json::from_str(&json).ok()
    }
}

fn scopes_into_key(scopes: &[&str]) -> String {
    let mut sorted: Vec<_> = scopes.to_vec();
    // Sort the scopes to ensure consistent key generation
    sorted.sort();

    let mut hasher = DefaultHasher::new();
    sorted.hash(&mut hasher);
    format!("token_{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod scopes_into_key_tests {
        use super::*;

        #[test]
        fn single_scope_produces_valid_key() {
            let scopes = ["single"];
            let key = scopes_into_key(&scopes);

            assert!(key.starts_with("token_"));
            assert!(key.len() > "token_".len());
        }

        #[test]
        fn same_scopes_produce_same_key() {
            let scopes = ["scope1", "scope2"];

            let key1 = scopes_into_key(&scopes);
            let key2 = scopes_into_key(&scopes);

            assert_eq!(key1, key2);
        }

        #[test]
        fn different_order_produces_same_key() {
            let scopes1 = ["scope1", "scope2"];
            let scopes2 = ["scope2", "scope1"];

            let key1 = scopes_into_key(&scopes1);
            let key2 = scopes_into_key(&scopes2);

            assert_eq!(key1, key2);
        }

        #[test]
        fn different_scopes_produce_different_keys() {
            let scopes1 = ["scope1"];
            let scopes2 = ["scope2"];

            let key1 = scopes_into_key(&scopes1);
            let key2 = scopes_into_key(&scopes2);

            assert_ne!(key1, key2);
        }

        #[test]
        fn empty_scopes_produce_valid_key() {
            let scopes: [&str; 0] = [];

            let key = scopes_into_key(&scopes);

            assert!(key.starts_with("token_"));
            assert!(key.len() > "token_".len());
        }
    }
}
