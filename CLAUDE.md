# Dawn - Personal Digital Assistant

Dawn is a **cross-platform native application** for managing personal schedules and tasks in one place.

## Vision

Based on GTD (Getting Things Done) philosophy, Dawn integrates **task management** and **calendar management** into a single application.

```
"Everything in one place" - Capture, Clarify, Organize, Reflect, Engage
```

## Core Philosophy

### GTD (Getting Things Done)

| Stage | Description | Dawn Feature |
|-------|-------------|--------------|
| **Capture** | Collect everything | Quick add (inbox) |
| **Clarify** | Process into actionable items | Task/Event distinction |
| **Organize** | Categorize by context, project | Tags, Projects, Contexts |
| **Reflect** | Periodic review | Weekly review, Dashboard |
| **Engage** | Execute | Today view, Next actions |

### Calendar Integration

Adding **time-based scheduling** to GTD's task management:

- **Tasks**: Actions that can be done anytime (due date optional)
- **Events**: Happenings at specific times (start/end time required)
- **Blocked Time**: Dedicated time slots for specific tasks (task + time block)

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

## Reference Projects

| Project | What to Learn |
|---------|---------------|
| [Taskwarrior](https://taskwarrior.org) | CLI UX, filter system, recurrence, urgency |
| [todo.txt](http://todotxt.org) | Simplicity, plain text philosophy |
| [GTD](https://gettingthingsdone.com) | Workflow methodology |

---

## Quick Commands

```bash
# Build & Test
cargo build
cargo t                    # Run all tests
cargo llvm-cov            # Coverage report

# CLI Development
cargo run -p dawn_cli -- <command>

# Lint
cargo clippy --all-targets
cargo fmt --check
```

---

## Related Documentation

All detailed guidelines are in the `.claude/` directory:

- **Architecture**: `skills/project-guidelines/`
- **Coding Style**: `rules/coding-style.md`, `skills/coding-standards/`
- **Rust Patterns**: `rules/rust.md`
- **Development Principles**: `rules/development-principles.md`
- **Testing**: `rules/testing.md`, `skills/tdd-workflow/`
- **Security**: `rules/security.md`, `skills/security-review/`
- **SQLite Patterns**: `skills/sqlite/`
- **Taskwarrior Reference**: `skills/taskwarrior/`

