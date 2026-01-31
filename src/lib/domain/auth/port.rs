//! Capability trait for services that support authentication

use std::future::Future;

/// Capability trait for services that support authentication
///
/// This trait defines the contract for services that can perform
/// authentication operations
pub trait AuthenticateService {
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}

/// Capability trait for providers that require authentication.
///
/// Separates authentication from data access to support both:
///
/// - Remote providers (e.g. Google Calendar, iCloud) that need authentication
/// - Local providers that don't require authentication
pub trait Authenticatable {
    /// Authenticates with the external service.
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}
