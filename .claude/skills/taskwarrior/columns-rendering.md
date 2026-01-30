# Taskwarrior Columns & Rendering

## Column Types

| Column | Description | Alignment |
|--------|-------------|-----------|
| `id` | Task ID | Right |
| `uuid` | UUID (short or full) | Left |
| `entry` | Creation date | Right |
| `modified` | Last modified | Right |
| `start` | Start time | Right |
| `end` | Completion time | Right |
| `due` | Due date | Right |
| `wait` | Wait until date | Right |
| `until` | Auto-delete date | Right |
| `scheduled` | Scheduled date | Right |
| `status` | Task status | Left |
| `description` | Task description | Left |
| `project` | Project name | Left |
| `priority` | H/M/L | Center |
| `tags` | Tags | Left |
| `depends` | Dependencies | Left |
| `urgency` | Calculated urgency | Right |
| `recur` | Recurrence interval | Left |

---

## Date Format Styles

| Style | Example | Description |
|-------|---------|-------------|
| `formatted` | `2024-01-15` | ISO date |
| `julian` | `2460325` | Julian day |
| `epoch` | `1705276800` | Unix timestamp |
| `iso` | `2024-01-15T00:00:00Z` | ISO 8601 |
| `age` | `2d` | Relative age |
| `relative` | `in 3d` | Relative to now |
| `remaining` | `3d` | Time remaining |
| `countdown` | `-2d` | Time until/since |

### Duration Formatting

| Range | Format |
|-------|--------|
| > 365 days | `Ny` (years) |
| > 30 days | `Nmo` (months) |
| > 7 days | `Nw` (weeks) |
| > 0 days | `Nd` (days) |
| > 0 hours | `Nh` (hours) |
| else | `Nmin` (minutes) |

---

## Report Configuration

Reports define which columns to display and how to sort/filter:

```
report.list.columns=id,project,priority,due,description
report.list.labels=ID,Project,Pri,Due,Description
report.list.sort=urgency-
report.list.filter=status:pending
```

### Default Reports

| Report | Columns | Sort |
|--------|---------|------|
| `list` | id, project, priority, due, description | urgency desc |
| `next` | id, project, priority, due, urgency, description | urgency desc |
| `all` | id, status, project, priority, due, description | entry asc |

---

## Sort Specifiers

| Format | Description |
|--------|-------------|
| `column+` | Ascending |
| `column-` | Descending |
| `col1+,col2-` | Multiple columns |

Sort key types:

- **String**: Alphabetical
- **Number**: Numeric
- **DateTime**: Chronological (None values last)
- **Priority**: H > M > L > None

---

## Custom Reports

Define custom reports in `.taskrc`:

```
report.urgent.description=Urgent tasks due soon
report.urgent.columns=id,priority,due,description
report.urgent.labels=ID,P,Due,Description
report.urgent.sort=due+,priority-
report.urgent.filter=+READY due.before:+7d
```

---

## Description Column Special Handling

The description column has special behaviors:

1. **Annotation indicator**: `[N]` suffix shows annotation count
2. **Truncation**: Ellipsis (`…`) when exceeding width
3. **Dependency indicator**: Can show blocked/blocking status

Example output:

```
Description
Buy groceries [2]
Review PR #123…
```

---

## Rust Implementation Notes

```rust
// Column trait for rendering
pub trait Column {
    fn name(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn render(&self, task: &Task, format: Option<&str>) -> String;
    fn alignment(&self) -> Alignment;
}

// Report configuration
pub struct Report {
    pub columns: Vec<String>,
    pub labels: Vec<String>,
    pub sort: Vec<SortSpec>,
    pub filter: Option<Filter>,
}

// Use tabled crate for table rendering
```
