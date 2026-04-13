/*
 * This lib target exists so that `cargo llvm-cov --workspace --lib` includes
 * this crate's CLI unit tests in coverage reports. Without a lib target,
 * `--lib` skips the crate entirely.
 */

mod arg;
mod cli;
mod handler;
mod table;

pub use cli::Cli;
