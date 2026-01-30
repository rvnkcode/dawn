use std::hash::{DefaultHasher, Hash, Hasher};

use async_trait::async_trait;
use keyring::Entry;
use yup_oauth2::storage::{TokenInfo, TokenStorage, TokenStorageError};

const SERVICE_NAME: &str = "dawn";

#[allow(dead_code)]
pub struct KeyringTokenStorage;

#[allow(dead_code)]
impl KeyringTokenStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TokenStorage for KeyringTokenStorage {
    /// Store the token associated with the given scopes in the keyring.
    async fn set(&self, scopes: &[&str], token: TokenInfo) -> Result<(), TokenStorageError> {
        let key = scopes_to_key(scopes);
        let entry = Entry::new(SERVICE_NAME, &key)
            .map_err(|e| TokenStorageError::Other(e.to_string().into()))?;
        let serialized_token = serde_json::to_string(&token)
            .map_err(|e| TokenStorageError::Other(e.to_string().into()))?;
        entry
            .set_password(&serialized_token)
            .map_err(|e| TokenStorageError::Other(e.to_string().into()))?;
        Ok(())
    }

    /// Retrieve the token associated with the given scopes from the keyring.
    async fn get(&self, scopes: &[&str]) -> Option<TokenInfo> {
        let key = scopes_to_key(scopes);
        let entry = Entry::new(SERVICE_NAME, &key).ok()?;
        let json = entry.get_password().ok()?;
        serde_json::from_str(&json).ok()
    }
}

/// Convert scopes to a deterministic key for keyring storage.
/// Scopes are sorted to ensure consistent key regardless of order.
fn scopes_to_key(scopes: &[&str]) -> String {
    let mut sorted: Vec<_> = scopes.to_vec();
    sorted.sort();

    let mut hasher = DefaultHasher::new();
    sorted.hash(&mut hasher);
    format!("token_{:x}", hasher.finish())
}
