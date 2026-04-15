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

## File Structure (Cargo Workspace)

```txt
project/
├── Cargo.toml                   # Workspace root
├── Cargo.lock
├── lib/                         # Library crate (dawn-lib)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Library root
│       ├── domain/              # Domain Core
│       └── outbound/            # Outbound Adapters (SQLite)
├── cli/                         # CLI crate (dawn-cli)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── cli.rs               # Clap parser
│   │   ├── arg.rs               # Argument types
│   │   └── handler.rs           # Command handlers
│   └── tests/                   # E2E tests
└── docs/                        # Documentation
```

Each inbound adapter is an independent workspace crate that depends on `dawn`.

## Testing Requirements

```sh
cargo test --workspace
```

## Deployment Workflow

### Pre-Deployment Checklist

- [ ] All tests passing locally
- [ ] `cargo build --workspace` succeeds
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
