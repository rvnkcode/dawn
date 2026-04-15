# GitHub Copilot Instructions

When reviewing pull requests for this project, apply the following guidelines.
Flag violations by severity: **CRITICAL** (must fix) > **HIGH** (should fix) > **MEDIUM** (prefer fix).

## Project Overview

Dawn is a cross-platform native application for personal task and calendar management, built on GTD philosophy.
Rust Cargo workspace with two crates: `lib/` (domain + outbound adapters) and `cli/` (inbound CLI adapter). SQLite for local storage. Phase 1 MVP scope is CLI only; TUI and GUI are planned but not yet implemented.

## Architecture

### Hexagonal Architecture

- **Layers**:
  - `lib/src/domain/` — Entities, value objects, domain services, port traits
  - `lib/src/outbound/` — Outbound adapters (SQLite, query builder, file I/O)
  - `cli/src/` — Inbound CLI adapter (drives the domain via ports exposed from `lib`)
- **Dependency direction**: Inbound (`cli`) and Outbound (`lib/outbound`) → Domain (`lib/domain`). Domain MUST NOT depend on adapters or frameworks
- External interactions go through Port traits; adapters implement ports, never the reverse
- Domain logic must be framework-free and testable in isolation

### SOLID & DDD in Practice

These principles are applied concretely in Dawn, not as abstract theory:

- `Task` is a domain entity with identity (`TaskId`). `TaskId`, `Description`, `Priority` are value objects — immutable, compared by value
- `Task` is the aggregate root; external code must not directly manipulate its internals (e.g., annotations, tags)
- Domain defines port traits (e.g., `TaskRepository`); `outbound/sqlite/` implements them. Domain never imports `rusqlite` or `clap`
- Each trait should be focused: separate `TaskReader` and `TaskWriter` over a monolithic `TaskStore`
- Extend behavior by adding new trait implementations, not by modifying existing ones
- Code naming MUST match Taskwarrior terminology (ubiquitous language)

## Coding Standards

### Immutability

ALWAYS create new objects, NEVER mutate existing ones:

```rust
// WRONG: mutates in-place
task.set_description(new_desc)
// RIGHT: returns new copy
task.with_description(new_desc)
```

Never expose `&mut` across module boundaries. Immutable data prevents hidden side effects and enables safe concurrency.

### File & Function Organization

- Many small files (200–400 lines typical, 800 max) over few large files
- Functions under 50 lines, no nesting deeper than 4 levels
- Organize by feature/domain, not by type
- Use `filename.rs` + `filename/` pattern, not `filename/mod.rs`

### Rust Idioms

- **Ownership**: Prefer `&str` over `String` in parameters; return `String` when ownership transfers
- **Borrowing**: Prefer `&`/`&mut` over `clone()`; avoid unnecessary allocations
- **Iterators**: Use `if let`/`while let` for optionals; `match` for exhaustive patterns; filter before `collect()`
- **Visibility**: Minimize `pub` exposure — only publish what's necessary
- **Naming**: `snake_case` functions/variables, `PascalCase` types, `UPPER_SNAKE_CASE` constants
- **Functions**: Verb-noun pattern (e.g., `create_task`, `parse_filter`)
- **Documentation**: `rustdoc` for public APIs; explain WHY, not WHAT

### Type Safety

- **Newtype pattern**: Wrap primitives for domain meaning (e.g., `struct TaskId(String)`)
- **Enums for variants**: Use enums instead of string constants or magic values
- **Bounded generics**: Prefer trait bounds over `dyn` when possible
- Derive `Debug` on all public types; `Clone`, `PartialEq` only when needed

### Error Handling & Safety

- `thiserror` for library errors, `anyhow` only in binary crates or tests
- **No `.unwrap()` or `.expect()` in production code** — propagate with `?`
- Handle errors explicitly at every level; never silently swallow errors
- User-facing errors must be friendly; internal errors must include context
- No `unsafe` blocks unless justified with a `// SAFETY:` comment

## Domain Rules (Taskwarrior)

### Key Design Decisions

- **Status is computed, not stored**: No `status` column in DB. Derived from other fields:
  - `deleted IS NOT NULL` → deleted
  - `end IS NOT NULL` → completed
  - `wait > now` → waiting
  - `recur IS NOT NULL` → recurring
  - Otherwise → pending
- **PK is nanoid (11 chars)**: Not UUID. Compact, URL-safe, sufficient collision resistance for local app. `CHECK (length(id) = 11)`
- **Virtual tags** (e.g., `+PENDING`, `+OVERDUE`, `+TODAY`, `+BLOCKING`): Calculated at runtime, never stored

### Ubiquitous Language

Use Taskwarrior terminology consistently: `Task`, `TaskId`, `Description`, `Project`, `Priority`, `Tag`, `Annotation`, `Recurrence`, `Filter`, `Report`.

## Input Validation & Security

- Validate all user input at system boundaries; fail fast with clear messages
- No hardcoded secrets (API keys, passwords, tokens)
- SQL injection prevention: parameterized queries only
- Error messages must not leak sensitive data
- Never trust external data (API responses, user input, file content)

## Configuration

- **`DAWN_DB_PATH`** — overrides the SQLite database file path. Takes precedence over the platform default (`~/.local/share/dawn/dawn.db` on Linux, `~/Library/Application Support/dawn/dawn.db` on macOS). Used by E2E tests for per-test isolation

## Testing

- **Minimum 80% test coverage** (measured via `cargo llvm-cov --workspace --lib`, excluding the E2E-only paths listed below)
- **TDD is mandatory**: RED → GREEN → REFACTOR (write test first, watch it fail, implement, pass, refactor)
- Unit tests for domain logic and outbound adapters (`#[cfg(test)]` modules in source files)
- Integration tests for adapters (`tests/` directory)
- **E2E-only layers** (no unit tests required; covered by CLI E2E tests):
  - `cli/src/cli.rs`, `cli/src/arg.rs`, `cli/src/handler.rs`
  - `lib/src/domain/task/service.rs`
- AAA pattern: Arrange → Act → Assert
- Descriptive test names reflecting the scenario
- Mock external services via ports, not by bypassing the architecture

## CLI Standards — POSIX Compliance

- **Exit codes**: `0` = success, `1` = runtime error, `2` = usage/argument error
- **Options**: Short (`-v`) and long (`--verbose`) forms; `--` terminates option parsing
- **Standard flags**: `--help` and `--version`
- **Streams**: `stdout` for output, `stderr` for errors/diagnostics, `stdin` for piped input
- **Signals**: Handle `SIGINT`/`SIGTERM` for graceful shutdown (flush buffers, release resources)

## Performance

- SELECT only needed columns, not `SELECT *`
- Use database indices for frequent queries
- No blocking I/O in async contexts
- Avoid unnecessary allocations; distinguish `iter()` vs `into_iter()`

## Review Checklist

### CRITICAL — Must Fix

- [ ] Immutability violated (mutation across module boundaries)
- [ ] Domain depends on adapter/framework code
- [ ] `.unwrap()` or `.expect()` in production code
- [ ] `unsafe` without `// SAFETY:` justification
- [ ] Hardcoded secrets or SQL injection risk
- [ ] Status stored as a column instead of computed

### HIGH — Should Fix

- [ ] SOLID principle violation (god struct, trait too broad, concrete dependency)
- [ ] Missing tests for new functionality or TDD not followed
- [ ] Incorrect error handling (`anyhow` in library code, swallowed errors)
- [ ] Type safety issue (primitive obsession, string constants instead of enums)
- [ ] CLI exit codes or signal handling incorrect
- [ ] Domain terminology mismatch (ubiquitous language)

### MEDIUM — Prefer Fix

- [ ] File >800 lines or function >50 lines
- [ ] Nesting deeper than 4 levels
- [ ] Unnecessary `clone()` or allocation
- [ ] Missing `rustdoc` on public API
- [ ] `SELECT *` or missing index consideration
