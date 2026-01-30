//! Google Calendar integration module
pub mod adapter;
pub use adapter::GoogleCalendarAdapter;
mod auth;
mod token_storage;
