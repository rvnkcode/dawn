# Taskwarrior Commands

> **How commands are resolved:** See [parsing-pipeline.md](parsing-pipeline.md) for the full argv → categorize → command flow. This document focuses on command taxonomy, capability flags, and default-command logic.

## Command Categories

### Metadata Commands

| Command       | Description                  |
| ------------- | ---------------------------- |
| `version`     | Show version info            |
| `commands`    | List all commands            |
| `show`        | Show configuration           |
| `udas`        | List user-defined attributes |
| `tags`        | List all tags                |
| `projects`    | List all projects            |
| `reports`     | List available reports       |
| `columns`     | List column types            |
| `diagnostics` | Show diagnostic info         |

### Report Commands

All reports are dispatched through `CmdCustom` with `filter=Yes, mods=No, misc=No`. See [capability flags](#capability-flags-by-command) for what this means.

| Command     | Description         | Default Filter              |
| ----------- | ------------------- | --------------------------- |
| `list`      | All pending tasks   | `status:pending`            |
| `next`      | Most urgent tasks   | `status:pending limit:page` |
| `all`       | All tasks           | (none)                      |
| `completed` | Completed tasks     | `status:completed`          |
| `deleted`   | Deleted tasks       | `status:deleted`            |
| `waiting`   | Waiting tasks       | `status:waiting`            |
| `recurring` | Recurring templates | `status:recurring`          |
| `blocked`   | Blocked tasks       | `+BLOCKED`                  |
| `blocking`  | Blocking tasks      | `+BLOCKING`                 |
| `overdue`   | Overdue tasks       | `+OVERDUE`                  |
| `ready`     | Ready tasks         | `+READY`                    |
| `active`    | Active tasks        | `+ACTIVE`                   |

### Operation Commands

| Command     | Description            |
| ----------- | ---------------------- |
| `add`       | Create new task        |
| `modify`    | Change existing tasks  |
| `done`      | Mark complete          |
| `delete`    | Mark deleted           |
| `start`     | Begin work             |
| `stop`      | Stop work              |
| `annotate`  | Add annotation         |
| `denotate`  | Remove annotation      |
| `duplicate` | Copy task              |
| `append`    | Append to description  |
| `prepend`   | Prepend to description |
| `edit`      | Edit in editor         |
| `undo`      | Undo last change       |
| `info`      | Show task details      |

For per-command capability flags (filter / mods / misc), see [capability flags table](#capability-flags-by-command).

### System Commands

| Command  | Description          |
| -------- | -------------------- |
| `sync`   | Sync with server     |
| `import` | Import JSON          |
| `export` | Export JSON          |
| `purge`  | Remove deleted tasks |

## Usage Patterns

```sh
# Add task with modifications
task add "Description" project:work +tag due:tomorrow

# Modify tasks matching filter
task project:work modify priority:H

# Complete task by ID
task 5 done

# Delete multiple tasks
task 5-10 delete

# List with additional filter
task list project:home

# Export filtered tasks
task status:pending export > tasks.json
```

## Argument Categorization

Argument interpretation depends on **command position** and **capability flags** declared by each command.

### Command Capability Flags

Each command declares three boolean flags:

| Flag                    | Description                                               |
| ----------------------- | --------------------------------------------------------- |
| `accepts_filter`        | Arguments before command are interpreted as filter        |
| `accepts_modifications` | Arguments after command are interpreted as modifications  |
| `accepts_miscellaneous` | Other arguments (rarely used)                             |

### Capability Flags by Command

Single source of truth for `_accepts_filter` / `_accepts_modifications` / `_accepts_miscellaneous`. Verified against the `Cmd*.cpp` constructors in the Taskwarrior source tree.

| Command     | Filter | Mods | Misc | Notes                                       |
| ----------- | ------ | ---- | ---- | ------------------------------------------- |
| `add`       | No     | Yes  | No   | All args become description/mods            |
| `modify`    | Yes    | Yes  | No   |                                             |
| `done`      | Yes    | Yes  | No   |                                             |
| `delete`    | Yes    | Yes  | No   | Trailing words → annotation                 |
| `start`     | Yes    | Yes  | No   | Trailing words → annotation                 |
| `stop`      | Yes    | Yes  | No   | Trailing words → annotation                 |
| `annotate`  | Yes    | Yes  | No   |                                             |
| `denotate`  | Yes    | No   | Yes  | Annotation text passed via `misc`           |
| `duplicate` | Yes    | Yes  | No   |                                             |
| `append`    | Yes    | Yes  | No   |                                             |
| `prepend`   | Yes    | Yes  | No   |                                             |
| `edit`      | Yes    | No   | No   |                                             |
| `undo`      | No     | No   | No   |                                             |
| `info`      | Yes    | No   | No   |                                             |
| `purge`     | Yes    | No   | No   | Trailing words narrow filter, not modify    |
| `export`    | Yes    | No   | Yes  | Misc used for output format flags           |
| Reports     | Yes    | No   | No   | `list`/`next`/`all`/etc. via `CmdCustom`    |

### Position-Based Interpretation

The `afterCommand` flag distinguishes before/after command:

```txt
task [BEFORE command] <command> [AFTER command]
         │                           │
         ▼                           ▼
    accepts_filter?            accepts_modifications?
         │                           │
         ▼                           ▼
      FILTER                    MODIFICATION
```

### Categorization Rules

| Filter | Mods | Before Command | After Command  |
| ------ | ---- | -------------- | -------------- |
| Yes    | Yes  | → FILTER       | → MODIFICATION |
| Yes    | No   | → FILTER       | → FILTER       |
| No     | Yes  | → MODIFICATION | → MODIFICATION |
| No     | No   | → Error        | → Error        |

### Dawn Handler Dispatch

When wiring a Dawn handler for a TW-parity command, pick the parser in `cli/src/filter.rs` based on the capability combo — **never by command intent.** `delete` and `purge` both "remove things" but have different `mods` flags and therefore different parsers.

| Filter | Mods | Dawn parser              | Post-args become              | Examples                                                |
| ------ | ---- | ------------------------ | ----------------------------- | ------------------------------------------------------- |
| Yes    | Yes  | `filter::parse_mutation` | annotation / description      | `delete`, `done`, `modify`, `start`, `stop`, `annotate` |
| Yes    | No   | `filter::parse_report`   | merged into filter (pre+post) | `purge`, `list`, `all`, `info`, `edit`                  |
| No     | Yes  | (custom; e.g. `add`)     | description / mods            | `add`                                                   |
| No     | No   | (none — no args needed)  | n/a                           | `undo`                                                  |

**Rule of thumb:** look up the command in [Capability Flags by Command](#capability-flags-by-command), then this table tells you exactly which `parse_*` function to call. Do not infer the parser from what the command "feels like."

### Examples

```sh
# modify: accepts_filter=Yes, accepts_modifications=Yes
task project:home modify due:tomorrow
#    └── FILTER ──┘        └── MOD ──┘

# add: accepts_filter=No, accepts_modifications=Yes
task project:home add Buy milk
#    └────────── MODIFICATION ──────────┘
# → description becomes "project:home Buy milk"

# list: accepts_filter=Yes, accepts_modifications=No
task project:home +urgent list
#    └────── FILTER ─────┘
# Arguments after command also treated as filter

# delete: accepts_filter=Yes, accepts_modifications=Yes
task 1-5 delete
#    └─┘ FILTER (task IDs 1 through 5)
# Trailing words after `delete` become an annotation on the deleted tasks.

# purge: accepts_filter=Yes, accepts_modifications=No
task status:deleted purge
#    └────── FILTER ─────┘
# Trailing words after `purge` are also treated as filter (narrowing).
```

### Plain Word Desugaring

Plain words in filter position are converted to `description ~ "word"`:

```sh
task shopping list
# "shopping" → description ~ "shopping"
# Searches tasks with "shopping" in description
```

Source: `~/Downloads/taskwarrior/src/CLI2.cpp` (`categorizeArgs()`, lines 978-1111)

## Default Command Resolution

When no explicit command keyword is present, Taskwarrior picks one based on whether the argument list contains an ID-like token.

Source: `~/Downloads/taskwarrior/src/CLI2.cpp` `defaultCommand()` (1791-1850).

### Decision Logic

```txt
found_command ← scan args for CMD tag
found_sequence ← scan args for Lexer::Type::number OR Lexer::Type::uuid
                 (NOTE: Lexer::Type::set is NOT included)

if found_command:
    use explicit command
elif found_sequence:
    inject "information"        → show task details
else:
    inject rc.default.command   → typically "next" (list view)
```

### The `set` vs `number` Trap

Because `set` tokens (`1,2,3` / `5-10`) are NOT counted as `found_sequence`, their default command differs from space-separated numbers:

| Input            | Tokens                   | found_sequence | Default command      |
| ---------------- | ------------------------ | -------------- | -------------------- |
| `task 1`         | `number(1)`              | true           | `information`        |
| `task 1 2`       | `number(1)` `number(2)`  | true           | `information`        |
| `task 1,2,3`     | `set(1,2,3)`             | false          | `rc.default.command` |
| `task 5-10`      | `set(5,10)`              | false          | `rc.default.command` |
| `task 1,2 3`     | `set(1,2)` `number(3)`   | true           | `information`        |
| `task 3 1,2`     | `number(3)` `set(1,2)`   | true           | `information`        |
| `task a1b2c3d4`  | `uuid(a1b2c3d4)`         | true           | `information`        |
| `task project:a` | `pair(project:a)`        | false          | `rc.default.command` |

**Rule of thumb:** If any token is a bare single ID/UUID, Taskwarrior assumes you want to inspect tasks → `information`. If only filters/sets are present, it runs the configured default report.

## Modification Syntax

| Syntax       | Description     | Example        |
| ------------ | --------------- | -------------- |
| `attr:value` | Set attribute   | `project:work` |
| `attr:`      | Clear attribute | `due:`         |
| `+tag`       | Add tag         | `+urgent`      |
| `-tag`       | Remove tag      | `-review`      |

### Attribute Shortcuts

| Full        | Short   |
| ----------- | ------- |
| `project`   | `pro`   |
| `priority`  | `pri`   |
| `scheduled` | `sched` |
| `depends`   | `dep`   |

## Confirmation Behavior

Operations that modify multiple tasks require confirmation:

```txt
$ task project:work delete
This will delete 15 tasks.
  a1b2c3d4 Review PR #123
  e5f6g7h8 Fix login bug
  ...
Proceed? (y/n)
```

Bypass with `rc.confirmation=off` or pipe `yes |`.

## Parsing Flow

```txt
CLI Input: task add "Buy milk" project:home +shopping due:tomorrow
                │
                ▼
┌───────────────────────────────────────┐
│            Clap Parser                │
│  Command::Add(args: ["Buy milk", ...])│
└───────────────────┬───────────────────┘
                    │
                    ▼
┌───────────────────────────────────────┐
│       Custom Modification Parser      │
│  description: "Buy milk"              │
│  mods: [project, tag, due]            │
└───────────────────┬───────────────────┘
                    │
                    ▼
┌───────────────────────────────────────┐
│         Command Execution             │
│  Create Task → Apply Mods → Save      │
└───────────────────────────────────────┘
```

## Modification Types

```rust
pub enum Modification {
    SetProject(Option<String>),
    SetPriority(Option<Priority>),
    SetDue(Option<DateTime<Utc>>),
    SetWait(Option<DateTime<Utc>>),
    SetScheduled(Option<DateTime<Utc>>),
    SetRecur(Option<RecurDuration>),
    AddTag(String),
    RemoveTag(String),
    AddAnnotation(String),
    AddDependency(Uuid),
    RemoveDependency(Uuid),
    SetUda(String, String),
}
```
