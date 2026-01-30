//! Google Calendar Adapter implementing CalendarRepository
use crate::domain::calendar::port::CalendarRepository;

use super::auth::{GoogleAuth, GoogleAuthError};

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
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.auth.token().await?;
        Ok(())
    }
}
