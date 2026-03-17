# Dawn - Personal Digital Assistant

Dawn is a **cross-platform native application** for managing personal schedules and tasks in one place.

## Vision

Based on GTD (Getting Things Done) philosophy, Dawn integrates **task management** and **calendar management** into a single application.

## Feature Scope

### Phase 1: Local Task Management (MVP)

- Taskwarrior-compatible task management
- CLI / TUI / GUI interfaces
- SQLite local storage

### Phase 2: Calendar Integration

- Event entity addition
- Calendar views (day/week/month)
- Time blocking

### Phase 3: External Sync

- Google Calendar integration
- iCloud Calendar integration
- Cross-device synchronization

## References

- [Taskwarrior](https://github.com/GothenburgBitFactory/taskwarrior)
- [todo.txt](https://github.com/todotxt/todo.txt)
- [GTD](https://gettingthingsdone.com)
- [Things](https://culturedcode.com/things/)

## Quick Commands

```sh
# Build & Test
cargo build --all-features
cargo test --all-features            # Run all tests
cargo llvm-cov --all-features --lib  # Coverage report (lib only, excludes bin)

# CLI Development
cargo run --features cli -- <command>

# Lint
cargo clippy --all-targets
cargo fmt --check
```

## Related Documentation

All detailed guidelines are in the `.claude/` directory:

- **Architecture**: `skills/project-guidelines/`
- **Coding Style**: `rules/common/coding-style.md`, `skills/coding-standards/`
- **Rust Patterns**: `rules/rust.md`
- **Development Principles**: `rules/development-principles.md`
- **Testing**: `rules/common/testing.md`
- **Security**: `rules/common/security.md`
- **Taskwarrior Reference**: `skills/taskwarrior/`
