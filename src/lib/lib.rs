//! # Dawn - Personal Digital Assistant
//!
//! Dawn is a cross-platform native application for managing personal schedules
//! and tasks in one place, based on the GTD (Getting Things Done) philosophy.
//!
//! ## Features
//!
//! - **Task Management**: Taskwarrior-compatible task handling
//! - **Calendar Integration**: Google Calendar events
//! - **Multiple Interfaces**: CLI, TUI, and GUI support

pub mod domain;
pub mod inbound;
pub mod outbound;

pub fn greet() -> &'static str {
    "Hello, world!"
}
