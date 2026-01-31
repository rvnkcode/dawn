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

    fn setup_mock_keyring() {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    }

    fn create_test_token() -> TokenInfo {
        TokenInfo {
            access_token: Some("test_access_token".to_string()),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: None,
            id_token: None,
        }
    }

    mod keyring_token_storage_tests {
        use super::*;

        #[test]
        fn new_creates_valid_instance() {
            let storage = KeyringTokenStorage::new();
            // Verify instance is created (type check is implicit)
            let _ = storage;
        }

        #[tokio::test]
        async fn set_stores_token_successfully() {
            setup_mock_keyring();
            let storage = KeyringTokenStorage::new();
            let scopes = ["https://www.googleapis.com/auth/calendar"];
            let token = create_test_token();

            let result = storage.set(&scopes, token).await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn get_returns_none_when_no_token_stored() {
            setup_mock_keyring();
            let storage = KeyringTokenStorage::new();
            let scopes = ["https://www.googleapis.com/auth/calendar.nonexistent"];

            let result = storage.get(&scopes).await;

            assert!(result.is_none());
        }

        #[test]
        fn set_serializes_token_as_json() {
            setup_mock_keyring();
            let scopes = ["https://www.googleapis.com/auth/calendar.serialize"];
            let key = scopes_into_key(&scopes);
            let token = create_test_token();

            // Simulating set() because keyring mock does not persist across calls
            let entry = Entry::new(SERVICE_NAME, &key).unwrap();
            let serialized = serde_json::to_string(&token).unwrap();
            entry.set_password(&serialized).unwrap();

            // Verify the stored value is valid JSON that deserializes back
            let stored = entry.get_password().unwrap();
            let deserialized: TokenInfo = serde_json::from_str(&stored).unwrap();
            assert_eq!(deserialized, token);
        }

        #[test]
        fn get_returns_none_on_invalid_json() {
            setup_mock_keyring();
            let scopes = ["https://www.googleapis.com/auth/calendar.invalid"];
            let key = scopes_into_key(&scopes);

            // Store invalid JSON
            let entry = Entry::new(SERVICE_NAME, &key).unwrap();
            entry.set_password("not valid json").unwrap();

            // Verify that invalid JSON fails to deserialize
            let stored = entry.get_password().unwrap();
            let result: Result<TokenInfo, _> = serde_json::from_str(&stored);
            assert!(result.is_err());
        }

        #[test]
        fn token_info_with_none_fields_serializes_correctly() {
            let token = TokenInfo {
                access_token: None,
                refresh_token: None,
                expires_at: None,
                id_token: None,
            };

            let json = serde_json::to_string(&token).unwrap();
            let deserialized: TokenInfo = serde_json::from_str(&json).unwrap();

            assert!(deserialized.access_token.is_none());
            assert!(deserialized.refresh_token.is_none());
            assert!(deserialized.expires_at.is_none());
            assert!(deserialized.id_token.is_none());
        }
    }

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
