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
