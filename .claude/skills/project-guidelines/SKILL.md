---
name: project-guidelines
description: Project specific skill
---
# Project Guidelines

## When to Use

Reference this skill when working on the specific project it's designed for.  
Project skills contain:

- Architecture overview
- File structure
- Testing requirements

## Architecture Overview

### Tech Stack

- **Language**: Rust, TypeScript
- **CLI**: Clap, Tabled
- **TUI**: Ratatui
- **GUI**: Tauri, Svelte
- **Database**: SQLite

### Services (Hexagonal Architecture)

```txt
┌─────────────────────────────────────────────────────────┐
│  Inbound: CLI | TUI | GUI                               │
└────────────────────────────┬────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────┐
│  Domain: Entities | Ports | Services                    │
└────────────────────────────┬────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────┐
│  Outbound: SQLite | Google Calendar | iCloud Calendar   │
└─────────────────────────────────────────────────────────┘
```

---

## File Structure (Single Crate with Feature Flags)

```txt
project/
├── Cargo.toml                    # Single crate manifest
├── Cargo.lock
├── src/
│   ├── bin/                      # Binary entry points
│   │   ├── cli/                  # CLI (feature: cli)
│   │   ├── tui/                  # TUI (feature: tui) [planned]
│   │   └── gui/                  # GUI (feature: gui) [planned]
│   └── lib/                      # Library code
│       ├── lib.rs                # Library root
│       ├── domain/               # Domain Core
│       ├── inbound/              # Inbound Adapters
│       │   └── cli.rs            # CLI adapter
│       └── outbound/             # Outbound Adapters
│           └── sqlite/           # SQLite adapter
├── tests/                        # Integration Tests
└── docs/                         # Documentation
```

### Feature Flags

```toml
[features]
cli = ["dep:clap"]      # CLI interface
tui = ["dep:ratatui"]   # TUI interface [planned]
gui = ["dep:tauri"]     # GUI interface [planned]
```

## Testing Requirements

```sh
cargo test --all-features
```

## Deployment Workflow

### Pre-Deployment Checklist

- [ ] All tests passing locally
- [ ] `cargo build --all-features` succeeds
- [ ] No hardcoded secrets
- [ ] Environment variables documented
- [ ] Database migrations ready

## Critical Rules

1. **No emojis** in code, comments, or documentation
2. **Immutability** - never mutate objects or arrays
3. **TDD** - write tests before implementation
4. **80% coverage** minimum
5. **Many small files** - 200-400 lines typical, 800 max
6. **Proper error handling** with `anyhow`, `thiserror` and `?`
7. **Input validation**

## Related Skills

- `/skills/coding-standards/` - General coding best practices
- `/skills/taskwarrior/` - Domain knowledge for Taskwarrior integration
