//! Authentication capability traits

use std::future::Future;

/// Service layer authentication capability.
pub trait AuthenticateService {
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}

/// Provider-level authentication capability.
///
/// Separates authentication from data access to support both:
/// - Remote providers (Google Calendar, iCloud) that need authentication
/// - Local providers that don't require authentication
pub trait Authenticatable {
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}
