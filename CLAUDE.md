# Dawn - Personal Digital Assistant

Dawn is a **cross-platform native application** for managing personal schedules and tasks in one place.

## Approach

- Think before acting. Read existing files before writing code.
- Be concise in output but thorough in reasoning.
- Prefer editing over rewriting whole files.
- Do not re-read files you have already read unless the file may have changed.
- Skip files over 100KB unless explicitly required.
- Suggest running /cost when a session is running long to monitor cache ratio.
- Recommend starting a new session when switching to an unrelated task.
- Test your code before declaring done.
- No sycophantic openers or closing fluff.
- Keep solutions simple and direct.
- User instructions always override this file.

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
cargo build --workspace
cargo test --workspace # Run all tests
# Coverage report (lib only, excludes bin)
cargo llvm-cov --workspace --lib --ignore-filename-regex='(cli/src/(cli|arg|error|handler)|lib/src/domain/task/service)\.rs$'

# CLI Development
cargo run -p dawn-cli -- <command>

# Lint
cargo clippy --workspace --all-targets
cargo fmt --check
```

## Environment Variables

- `DAWN_DB_PATH` — override the SQLite database file path. Takes precedence over the platform default (`~/.local/share/dawn/dawn.db` on Linux, `~/Library/Application Support/dawn/dawn.db` on macOS). Intended for E2E tests (isolating per-test databases) and advanced users.

## Related Documentation

All detailed guidelines are in the `.claude/` directory:

- **Architecture**: `skills/project-guidelines/`
- **Coding Style**: `rules/common/coding-style.md`, `rules/rust/coding-style.md`, `skills/coding-standards/`
- **Rust Patterns**: `skills/rust-patterns/`
- **Development Principles**: `rules/development-principles.md`
- **Testing**: `rules/common/testing.md`, `rules/rust/testing.md`, `skills/rust-testing/`
- **Security**: `rules/common/security.md`, `rules/rust/security.md`
- **Taskwarrior Reference**: `skills/taskwarrior/`
