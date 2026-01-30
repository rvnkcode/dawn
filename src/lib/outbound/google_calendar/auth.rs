use yup_oauth2::authenticator::DefaultAuthenticator;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use super::token_storage::KeyringTokenStorage;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
// refs: https://developers.google.com/workspace/calendar/api/auth
const CALENDAR_SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";

pub struct GoogleAuth {
    authenticator: DefaultAuthenticator,
}

// TODO: Automatically open browser
impl GoogleAuth {
    pub async fn new() -> Result<Self, GoogleAuthError> {
        let client_id = dotenvy::var("GOOGLE_CLIENT_ID")?;
        let client_secret = dotenvy::var("GOOGLE_CLIENT_SECRET")?;
        let secret = yup_oauth2::ApplicationSecret {
            client_id,
            client_secret,
            auth_uri: AUTH_URL.to_string(),
            token_uri: TOKEN_URL.to_string(),
            ..Default::default()
        };
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .with_storage(Box::new(KeyringTokenStorage::new()))
                .build()
                .await?;

        Ok(Self {
            authenticator: auth,
        })
    }

    pub async fn token(&self) -> Result<(), GoogleAuthError> {
        self.authenticator.token(&[CALENDAR_SCOPE]).await?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GoogleAuthError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] dotenvy::Error),
    #[error("Authenticator error: {0}")]
    Authenticator(#[from] std::io::Error),
    #[error("Token error: {0}")]
    Token(#[from] yup_oauth2::Error),
}
