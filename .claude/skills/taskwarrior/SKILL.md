---
name: Taskwarrior
---

# Taskwarrior Skill

When implementing Taskwarrior features in Dawn, refer to these resources and documentation.

## External References

- Source Code: `~/Downloads/taskwarrior` (C++ implementation)
- Documentation: <https://taskwarrior.org/docs/>
- Man Pages: `man task` for command reference
- Man Pages Source: `~/Downloads/taskwarrior/doc/man/`
- Reference PDF: `~/Downloads/taskwarrior/doc/ref/task-ref.pdf`

---

## Skill Documentation

| Document | Description |
|----------|-------------|
| [data-model.md](data-model.md) | Task entity, attributes, status, virtual tags |
| [filter-system.md](filter-system.md) | Filter expressions, operators, parsing |
| [commands.md](commands.md) | Command categories and patterns |
| [columns-rendering.md](columns-rendering.md) | Column types and rendering |
| [recurrence.md](recurrence.md) | Recurring task mechanism |
| [rust-patterns.md](rust-patterns.md) | C++ to Rust conversion patterns |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Taskwarrior Components                       │
├─────────────────────────────────────────────────────────────────┤
│  C++ (Original)              │  Rust (Dawn)                     │
├──────────────────────────────┼──────────────────────────────────┤
│  Task class                  │  Task struct                     │
│  Context class               │  AppContext struct               │
│  Command hierarchy           │  Clap subcommands                │
│  Filter class                │  Filter enum + parser            │
│  Column hierarchy            │  Column trait + impls            │
│  TDB2 (file storage)         │  SQLite adapter                  │
│  Lexer/Parser (commands)     │  Clap derive macros              │
│  Lexer/Parser (filters)      │  nom / pest parser               │
└──────────────────────────────┴──────────────────────────────────┘
```

### Parsing Layers

| Layer | Tool | Example |
|-------|------|---------|
| CLI commands | Clap | `task add`, `task list`, `task 5 done` |
| Filters & modifications | Custom parser | `project:work +urgent due:tomorrow` |

---

## Quick Reference

### Task Status

| Status | Description | Transitions |
|--------|-------------|-------------|
| `pending` | Default for new tasks | -> completed, deleted, waiting |
| `completed` | Finished tasks | (terminal) |
| `deleted` | Removed tasks | (terminal) |
| `waiting` | Hidden until wait date | -> pending (automatic) |
| `recurring` | Template for recurring tasks | generates pending instances |

### Priority

| Value | Meaning | Urgency Coefficient |
|-------|---------|---------------------|
| `H` | High | +6.0 |
| `M` | Medium | +3.9 |
| `L` | Low | +1.8 |
| (none) | No priority | 0.0 |

### Common Virtual Tags

| Tag | Description |
|-----|-------------|
| `+PENDING` | status == pending |
| `+COMPLETED` | status == completed |
| `+DELETED` | status == deleted |
| `+WAITING` | status == waiting |
| `+READY` | pending, not blocked, not waiting |
| `+ACTIVE` | has start time, not ended |
| `+SCHEDULED` | has scheduled date |
| `+UNTIL` | has until date |
| `+ANNOTATED` | has annotations |
| `+TAGGED` | has at least one tag |
| `+PARENT` | is a recurring parent |
| `+CHILD` | is a recurring instance |
| `+BLOCKING` | blocks other tasks |
| `+BLOCKED` | blocked by other tasks |
| `+OVERDUE` | past due date |
| `+TODAY` | due today |
| `+TOMORROW` | due tomorrow |
| `+WEEK` | due within 7 days |
| `+MONTH` | due within 30 days |
| `+QUARTER` | due within 90 days |
| `+YEAR` | due within 365 days |

### Urgency Formula

```
urgency = Σ (coefficient × condition)

Default coefficients:
  priority.H      =  6.0
  priority.M      =  3.9
  priority.L      =  1.8
  project         =  1.0
  active          =  4.0
  scheduled       =  5.0
  age             =  2.0 (scaled by age)
  annotations     =  1.0
  tags            =  1.0
  due             =  12.0 (scaled by proximity)
  blocking        =  8.0
  blocked         = -5.0
```
