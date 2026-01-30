use super::auth::{GoogleAuth, GoogleAuthError};

#[allow(dead_code)]
pub struct GoogleCalendarAdapter {
    auth: GoogleAuth,
}

#[allow(dead_code)]
impl GoogleCalendarAdapter {
    pub async fn new() -> Result<Self, GoogleAuthError> {
        Ok(Self {
            auth: GoogleAuth::new().await?,
        })
    }

    // TODO: Implement port trait
    async fn authenticate(&self) -> Result<(), GoogleAuthError> {
        self.auth.get_token().await?;
        Ok(())
    }
}
