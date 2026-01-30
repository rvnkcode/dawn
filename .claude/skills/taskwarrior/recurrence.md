# Taskwarrior Recurrence

## Overview

Recurring tasks use a parent-child model:

```
PARENT (Template)              CHILDREN (Instances)
status: recurring              status: pending
recur: weekly                  parent: <parent_uuid>
due: 2024-01-15                due: 2024-01-15
                               due: 2024-01-22
                               due: 2024-01-29
                               ...
```

- Parent has `status:recurring` and never completes
- Children are generated automatically with `status:pending`
- Each child references parent via `parent` attribute

---

## Recurrence Frequencies

### Named Frequencies

| Format | Description |
|--------|-------------|
| `daily` | Every day |
| `weekdays` | Monday through Friday |
| `weekly` | Every 7 days |
| `biweekly` | Every 14 days |
| `monthly` | Every month (same day) |
| `bimonthly` | Every 2 months |
| `quarterly` | Every 3 months |
| `semiannual` | Every 6 months |
| `annual` / `yearly` | Every year |

### Custom Intervals

| Format | Example | Description |
|--------|---------|-------------|
| `Nd` | `3d` | Every N days |
| `Nw` | `2w` | Every N weeks |
| `Nmo` | `4mo` | Every N months |
| `Nq` | `2q` | Every N quarters |
| `Ny` | `5y` | Every N years |

---

## Creating Recurring Tasks

```bash
# Weekly task
task add "Team standup" recur:weekly due:monday

# Every 3 days
task add "Water plants" recur:3d due:today

# Monthly on the 15th
task add "Pay rent" recur:monthly due:2024-01-15
```

Requirements:

- Must have `recur` attribute
- Must have `due` attribute
- Optional: `until` to limit recurrence end date

---

## Instance Generation

Taskwarrior generates instances based on `recurrence.limit` (default: 1):

1. When a child is completed, next instance is created
2. Generation happens at task list/sync time
3. Instances have same attributes as parent (project, tags, priority)

### Until Date

```bash
# Recurring until end of year
task add "Weekly report" recur:weekly due:friday until:eoy
```

After `until` date, no more instances are generated.

---

## Modifying Recurring Tasks

| Action on Parent | Effect |
|------------------|--------|
| Modify description | Updates future instances |
| Modify project/tags | Updates future instances |
| Change `recur` | Changes interval |
| Change `due` | Changes base date |
| Delete | Deletes parent + all children |

| Action on Child | Effect |
|-----------------|--------|
| Complete | Marks this instance done, generates next |
| Modify | Changes this instance only |
| Delete | Deletes this instance only |

---

## Virtual Tags

| Tag | Description |
|-----|-------------|
| `+PARENT` | Is a recurring parent (status:recurring) |
| `+CHILD` | Is a recurring child (has parent UUID) |

```bash
# List all recurring templates
task +PARENT list

# List all recurring instances
task +CHILD list
```

---

## Date Calculation for Months

When recurring monthly, Taskwarrior handles month-end dates:

| Base Date | Next Month |
|-----------|------------|
| Jan 31 | Feb 28/29 |
| Jan 30 | Feb 28/29 |
| Jan 29 | Feb 28/29 |
| Jan 28 | Feb 28 |

Day is capped to last day of month if overflow.

---

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `recurrence.limit` | 1 | Max pending instances |
| `recurrence.indicator` | yes | Show indicator in reports |
| `recurrence.confirmation` | yes | Confirm delete recurring |

---

## Rust Implementation Notes

```rust
#[derive(Debug, Clone, Copy)]
pub enum RecurDuration {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Quarters(u32),
    Years(u32),
    Weekdays,
}

impl RecurDuration {
    pub fn next_occurrence(&self, from: DateTime<Utc>) -> DateTime<Utc>;
}

impl FromStr for RecurDuration {
    // Parse "weekly", "3d", "2mo", etc.
}
```
