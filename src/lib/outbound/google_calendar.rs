//! Google Calendar integration

mod auth;
mod token_storage;

use crate::{
    domain::calendar::port::CalendarRepository,
    outbound::google_calendar::auth::{GoogleAuth, GoogleAuthError},
};

/// Implements [`CalendarRepository`] port via Google Calendar API
pub struct GoogleCalendarAdapter {
    auth: GoogleAuth,
}

impl GoogleCalendarAdapter {
    pub async fn new() -> Result<Self, GoogleAuthError> {
        Ok(Self {
            auth: GoogleAuth::new().await?,
        })
    }
}

impl CalendarRepository for GoogleCalendarAdapter {
    /// Authenticate Google OAuth2 to access the Calendar API
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.auth.token().await?;
        Ok(())
    }
}
