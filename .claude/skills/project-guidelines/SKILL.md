# Project Guidelines

## When to Use

Reference this skill when working on the specific project it's designed for.  
Project skills contain:

- Architecture overview
- File structure

---

## Architecture Overview

### Tech Stack

- **Language**: Rust, TypeScript
- **CLI**: Clap, Tabled
- **TUI**: Ratatui
- **GUI**: Tauri, Svelte
- **Database**: SQLite

### Services (Hexagonal Architecture)

```
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

```
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
│       ├── domain/               # Domain Core [planned]
│       ├── inbound/              # Inbound Adapters
│       │   └── cli.rs            # CLI adapter
│       └── outbound/             # Outbound Adapters [planned]
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

---

## Testing Requirements

```sh
cargo t
```

---

## Deployment Workflow

### Pre-Deployment Checklist

- [ ] All tests passing locally
- [ ] `cargo build` succeeds
- [ ] No hardcoded secrets
- [ ] Environment variables documented
- [ ] Database migrations ready

## Critical Rules

1. **No emojis** in code, comments, or documentation
2. **TDD** - write tests before implementation
3. **80% coverage** minimum
4. **Many small files** - 200-400 lines typical, 800 max
5. **Proper error handling** with `thiserror`, `anyhow` and `?`
6. **Input validation**

## Related Skills

- `/skills/coding-standards/` - General coding best practices
- `/skills/taskwarrior/` - Domain knowledge for Taskwarrior integration
- `/skills/tdd-workflow/` - Test-driven development methodology
