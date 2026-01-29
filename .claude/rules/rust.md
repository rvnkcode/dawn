# Rust Specific Rules

- Avoid unnecessary `clone()`
- Avoid `unwrap()` → Use `expect("description")` or `?`
- Avoid magic numbers → Define constants with meaningful names
- Use `match` for exhaustive pattern matching
- Use `if let`, `while let` for optional values
- Use `&str` vs `String` appropriately
- Prefer borrowing (`&`, `&mut`) when possible
- Minimize `pub` exposure (only publish what's necessary)
- Follow Rust naming conventions (snake_case for variables/functions, CamelCase for types/traits)
- Use `#[derive(Debug, PartialEq)]` where applicable
- Single responsibility principle for functions and modules

## Error Handling

- Define custom error types for domain-specific errors (`thiserror`)
- Prefer `?` for propagating errors

## Module Organization

 - Use `filename.rs` + `filename/` pattern instead of `filename/mod.rs`

## Performance

- Avoid unnecessary allocations
- Distinguish `iter()` vs `into_iter()`
- Filter before `collect()`

## Documentation

- Use `///` doc comments for public APIs

