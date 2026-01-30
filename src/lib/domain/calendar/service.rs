//! Calendar domain service implementation
//!
//! This module contains the concrete implementation of [`CalendarService`],
//! which orchestrates calendar operations using a [`CalendarRepository`].
use super::port::{CalendarRepository, CalendarService};

/// Domain service for calendar operations.
///
/// Implements [`CalendarService`] inbound port by delegating to a
/// [`CalendarRepository`] outbound port.
///
/// # Type Parameters
///
/// * `C` - A type implementing [`CalendarRepository`] for external calendar access
///
/// # Example
///
/// ```ignore
/// let repo = GoogleCalendarRepository::new(credentials);
/// let service = Service::new(repo);
/// service.authenticate().await?;
/// ```
pub struct Service<C: CalendarRepository> {
    repo: C,
}

impl<C: CalendarRepository> Service<C> {
    /// Creates a new calendar service with the given repository.
    pub fn new(repo: C) -> Self {
        Self { repo }
    }
}

impl<C: CalendarRepository> CalendarService for Service<C> {
    /// Authenticates with the calendar provider.
    ///
    /// Delegates to the underlying repository's authentication.
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.repo.authenticate().await
    }
}
