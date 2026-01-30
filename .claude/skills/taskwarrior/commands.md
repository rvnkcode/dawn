# Taskwarrior Commands

## Command Categories

### Metadata Commands

| Command | Description |
|---------|-------------|
| `version` | Show version info |
| `commands` | List all commands |
| `show` | Show configuration |
| `udas` | List user-defined attributes |
| `tags` | List all tags |
| `projects` | List all projects |
| `reports` | List available reports |
| `columns` | List column types |
| `diagnostics` | Show diagnostic info |

### Report Commands

| Command | Description | Default Filter |
|---------|-------------|----------------|
| `list` | All pending tasks | `status:pending` |
| `next` | Most urgent tasks | `status:pending limit:page` |
| `all` | All tasks | (none) |
| `completed` | Completed tasks | `status:completed` |
| `deleted` | Deleted tasks | `status:deleted` |
| `waiting` | Waiting tasks | `status:waiting` |
| `recurring` | Recurring templates | `status:recurring` |
| `blocked` | Blocked tasks | `+BLOCKED` |
| `blocking` | Blocking tasks | `+BLOCKING` |
| `overdue` | Overdue tasks | `+OVERDUE` |
| `ready` | Ready tasks | `+READY` |
| `active` | Active tasks | `+ACTIVE` |

### Operation Commands

| Command | Accepts Filter | Accepts Mods | Description |
|---------|---------------|--------------|-------------|
| `add` | No | Yes | Create new task |
| `modify` | Yes | Yes | Change existing tasks |
| `done` | Yes | Yes | Mark complete |
| `delete` | Yes | No | Mark deleted |
| `start` | Yes | No | Begin work |
| `stop` | Yes | No | Stop work |
| `annotate` | Yes | No | Add annotation |
| `denotate` | Yes | No | Remove annotation |
| `duplicate` | Yes | Yes | Copy task |
| `append` | Yes | No | Append to description |
| `prepend` | Yes | No | Prepend to description |
| `edit` | Yes | No | Edit in editor |
| `undo` | No | No | Undo last change |

### System Commands

| Command | Description |
|---------|-------------|
| `sync` | Sync with server |
| `import` | Import JSON |
| `export` | Export JSON |
| `purge` | Remove deleted tasks |

---

## Usage Patterns

```bash
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

---

## Argument Categorization

Argument interpretation depends on **command position** and **capability flags** declared by each command.

### Command Capability Flags

Each command declares three boolean flags:

| Flag | Description |
|------|-------------|
| `accepts_filter` | Arguments before command are interpreted as filter |
| `accepts_modifications` | Arguments after command are interpreted as modifications |
| `accepts_miscellaneous` | Other arguments (rarely used) |

### Flag Settings by Command

| Command | Filter | Mods | Misc |
|---------|--------|------|------|
| `add` | No | Yes | No |
| `modify` | Yes | Yes | No |
| `done` | Yes | Yes | No |
| `delete` | Yes | No | No |
| `start` | Yes | No | No |
| `stop` | Yes | No | No |
| `list` | Yes | No | No |
| `annotate` | Yes | Yes | No |
| `append` | Yes | Yes | No |
| `prepend` | Yes | Yes | No |
| `duplicate` | Yes | Yes | No |
| `info` | Yes | No | No |
| `export` | Yes | No | No |

### Position-Based Interpretation

The `afterCommand` flag distinguishes before/after command:

```
task [BEFORE command] <command> [AFTER command]
         │                           │
         ▼                           ▼
    accepts_filter?            accepts_modifications?
         │                           │
         ▼                           ▼
      FILTER                    MODIFICATION
```

### Categorization Rules

| Filter | Mods | Before Command | After Command |
|--------|------|----------------|---------------|
| Yes | Yes | → FILTER | → MODIFICATION |
| Yes | No | → FILTER | → FILTER |
| No | Yes | → MODIFICATION | → MODIFICATION |
| No | No | → Error | → Error |

### Examples

```bash
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

# delete: accepts_filter=Yes, accepts_modifications=No
task 1-5 delete
#    └─┘ FILTER (task IDs 1 through 5)
```

### Plain Word Desugaring

Plain words in filter position are converted to `description ~ "word"`:

```bash
task shopping list
# "shopping" → description ~ "shopping"
# Searches tasks with "shopping" in description
```

Source: `~/Downloads/taskwarrior/src/CLI2.cpp` (`categorizeArgs()`, lines 978-1111)

---

## Modification Syntax

| Syntax | Description | Example |
|--------|-------------|---------|
| `attr:value` | Set attribute | `project:work` |
| `attr:` | Clear attribute | `due:` |
| `+tag` | Add tag | `+urgent` |
| `-tag` | Remove tag | `-review` |

### Attribute Shortcuts

| Full | Short |
|------|-------|
| `project` | `pro` |
| `priority` | `pri` |
| `scheduled` | `sched` |
| `depends` | `dep` |

---

## Confirmation Behavior

Operations that modify multiple tasks require confirmation:

```
$ task project:work delete
This will delete 15 tasks.
  a1b2c3d4 Review PR #123
  e5f6g7h8 Fix login bug
  ...
Proceed? (y/n)
```

Bypass with `rc.confirmation=off` or pipe `yes |`.

---

## Clap CLI Structure

```rust
#[derive(Parser)]
#[command(name = "task")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Add(AddArgs),
    #[command(alias = "mod")]
    Modify(ModifyArgs),
    Done(FilterArgs),
    Delete(FilterArgs),
    List(ListArgs),
    // ...
}

#[derive(Parser)]
pub struct AddArgs {
    /// Description and modifications: "Buy milk project:home +shopping"
    #[arg(trailing_var_arg = true, required = true)]
    pub args: Vec<String>,
}

#[derive(Parser)]
pub struct ModifyArgs {
    /// Filter expression
    #[arg(required = true)]
    pub filter: String,
    /// Modifications to apply
    #[arg(trailing_var_arg = true)]
    pub modifications: Vec<String>,
}
```

---

## Parsing Flow

```
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

---

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
