/*
 * This lib target exists so that `cargo llvm-cov --workspace --lib` includes
 * CLI unit tests in coverage reports. Removing it would cause a compile error
 * because `SQLite::new()` is `#[cfg(not(coverage))]`.
 */

mod arg;
mod cli;
mod handler;

pub use cli::Cli;
