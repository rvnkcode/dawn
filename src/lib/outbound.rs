//! Driven adapters (repositories, external APIs)
//!
//! This module contains the implementations of driven adapters that interact with external
//! systems, such as databases, third-party APIs, or other services.

#[cfg(feature = "google-calendar")]
pub mod google_calendar;
