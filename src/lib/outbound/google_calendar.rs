//! Google Calendar integration
//!
//! API documentation: <https://developers.google.com/calendar/api/v3/reference/>

mod auth;
mod token_storage;

#[cfg(not(coverage))]
use crate::domain::auth::port::Authenticatable;
#[cfg(not(coverage))]
use crate::outbound::google_calendar::auth::{GoogleAuth, GoogleAuthError};

/// Implementation of [`Authenticatable`] for Google Calendar API
#[cfg(not(coverage))]
pub struct GoogleCalendarAdapter {
    auth: GoogleAuth,
}

#[cfg(not(coverage))]
impl GoogleCalendarAdapter {
    pub async fn new() -> Result<Self, GoogleAuthError> {
        Ok(Self {
            auth: GoogleAuth::new().await?,
        })
    }
}

#[cfg(not(coverage))]
impl Authenticatable for GoogleCalendarAdapter {
    /// Authenticate Google OAuth2 to access the Calendar API
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.auth.token().await?;
        Ok(())
    }
}
