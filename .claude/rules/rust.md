# Rust Specific Rules

## Ownership & Borrowing

- Prefer `&str` over `String` in function parameters; return `String` when ownership transfers
- Prefer borrowing (`&`, `&mut`) when possible
- Avoid unnecessary `clone()`
- Distinguish `iter()` vs `into_iter()`

## Idiomatic Rust

- Use `if let`, `while let` for optional values
- Use `match` for exhaustive pattern matching
- Filter before `collect()`
- Avoid unnecessary allocations
- Derive `Debug` on all public types; derive `Clone`, `PartialEq` only when needed

## Code Style

- Use `filename.rs` + `filename/` pattern instead of `filename/mod.rs`
- Types: PascalCase, functions/variables: snake_case, constants: UPPER_SNAKE_CASE
- Minimize `pub` exposure (only publish what's necessary)

## Error Handling

- Use `thiserror` for library errors, `anyhow` only in binary crates or tests
- No `.unwrap()` or `.expect()` in production code — propagate errors with `?`

## Safety

- No `unsafe` blocks unless justified with a `// SAFETY:` comment

## Testing

- Unit tests in `#[cfg(test)]` modules within each source file
- Integration tests in `tests/` directory
- Mock external services with `mockall` or `wiremock`
