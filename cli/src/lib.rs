/*
 * This lib target exists so that `cargo llvm-cov --workspace --lib` includes
 * this crate's CLI unit tests in coverage reports. Without a lib target,
 * `--lib` skips the crate entirely.
 */

mod arg;
mod cli;
mod error;
mod filter;
mod handler;
mod table;
#[cfg(test)]
mod test_helper;

pub use cli::Cli;
pub use error::CliError;
