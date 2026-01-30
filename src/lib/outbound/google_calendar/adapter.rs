//! Google Calendar adapter implementation
//!
//! This module provides the outbound adapter for Google Calendar API,
//! implementing the [`CalendarRepository`] port from the domain layer.
use crate::domain::calendar::port::CalendarRepository;

use super::auth::{GoogleAuth, GoogleAuthError};

/// Outbound adapter for Google Calendar API.
///
/// Implements [`CalendarRepository`] to provide Google Calendar
/// as an external calendar provider.
///
/// # Example
///
/// ```ignore
/// let adapter = GoogleCalendarAdapter::new().await?;
/// adapter.authenticate().await?;
/// ```
pub struct GoogleCalendarAdapter {
    auth: GoogleAuth,
}

impl GoogleCalendarAdapter {
    /// Creates a new adapter with Google OAuth credentials.
    ///
    /// # Errors
    ///
    /// Returns [`GoogleAuthError`] if credential loading or OAuth setup fails.
    pub async fn new() -> Result<Self, GoogleAuthError> {
        Ok(Self {
            auth: GoogleAuth::new().await?,
        })
    }
}

impl CalendarRepository for GoogleCalendarAdapter {
    /// Authenticates with Google Calendar API.
    ///
    /// Retrieves or refreshes the OAuth token as needed.
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.auth.token().await?;
        Ok(())
    }
}
