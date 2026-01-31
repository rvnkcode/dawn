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

#[cfg(test)]
mod tests {
    use super::*;

    mod google_auth_error {
        use super::*;

        #[test]
        fn from_dotenvy_error_converts_correctly() {
            let dotenv_err = dotenvy::Error::LineParse("test".to_string(), 1);
            let auth_err: GoogleAuthError = dotenv_err.into();

            assert!(matches!(auth_err, GoogleAuthError::EnvVar(_)));
        }

        #[test]
        fn from_io_error_converts_correctly() {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
            let auth_err: GoogleAuthError = io_err.into();

            assert!(matches!(auth_err, GoogleAuthError::Authenticator(_)));
        }

        #[test]
        fn env_var_error_display_includes_prefix() {
            let dotenv_err = dotenvy::Error::LineParse("test".to_string(), 1);
            let auth_err: GoogleAuthError = dotenv_err.into();

            assert!(
                auth_err
                    .to_string()
                    .starts_with("Environment variable error:")
            );
        }

        #[test]
        fn authenticator_error_display_includes_prefix() {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
            let auth_err: GoogleAuthError = io_err.into();

            assert!(auth_err.to_string().starts_with("Authenticator error:"));
        }
    }

    mod constants {
        use super::*;

        #[test]
        fn auth_url_uses_https() {
            assert!(AUTH_URL.starts_with("https://"));
        }

        #[test]
        fn auth_url_points_to_google() {
            assert!(AUTH_URL.contains("google.com"));
        }

        #[test]
        fn token_url_uses_https() {
            assert!(TOKEN_URL.starts_with("https://"));
        }

        #[test]
        fn token_url_points_to_google() {
            assert!(TOKEN_URL.contains("googleapis.com"));
        }

        #[test]
        fn calendar_scope_uses_https() {
            assert!(CALENDAR_SCOPE.starts_with("https://"));
        }

        #[test]
        fn calendar_url_points_to_google() {
            assert!(CALENDAR_SCOPE.contains("googleapis.com"));
        }

        #[test]
        fn calendar_scope_is_readonly() {
            assert!(CALENDAR_SCOPE.contains("readonly"));
        }

        #[test]
        fn calendar_scope_is_for_calendar_api() {
            assert!(CALENDAR_SCOPE.contains("calendar"));
        }
    }
}
