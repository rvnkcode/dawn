//! Driving adapters (CLI, TUI, GUI)
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub use cli::Cli;
