//! Port traits for calendar domain
use std::future::Future;

/// Inbound port for calendar operations.
///
/// Defines use cases that external adapters (e.g. CLI) can invoke.
pub trait CalendarService {
    /// Authenticates with the calendar provider.
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}

/// Outbound port for calendar data access.
///
/// Abstracts external calendar providers (Google Calendar, iCloud, etc.).
pub trait CalendarRepository {
    /// Authenticates with the external calendar service.
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}
