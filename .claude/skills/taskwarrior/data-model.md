# Taskwarrior Data Model

## Task Entity

The Task is the core entity in Taskwarrior. Each task has a unique UUID and various attributes.

### Core Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `uuid` | UUID | Yes | Unique identifier (v4) |
| `entry` | DateTime | Yes | Creation timestamp |
| `status` | Status | Yes | Current state |
| `description` | String | Yes | Task description |
| `modified` | DateTime | No | Last modification time |
| `start` | DateTime | No | When work began |
| `end` | DateTime | No | Completion/deletion time |
| `due` | DateTime | No | Due date |
| `wait` | DateTime | No | Hidden until this date |
| `until` | DateTime | No | Auto-delete after this date |
| `scheduled` | DateTime | No | Planned start date |
| `recur` | Duration | No | Recurrence interval |
| `parent` | UUID | No | Parent recurring task |
| `project` | String | No | Project name (hierarchical) |
| `priority` | Priority | No | H, M, L, or none |
| `depends` | Vec<UUID> | No | Dependency UUIDs |
| `tags` | Vec<String> | No | User tags |
| `annotations` | Vec<Annotation> | No | Timestamped notes |

### User Defined Attributes (UDA)

Tasks can have custom attributes defined in configuration, stored as key-value pairs.

---

## Status

| Status | Description |
|--------|-------------|
| `pending` | Default for new tasks |
| `completed` | Finished tasks |
| `deleted` | Removed tasks |
| `waiting` | Hidden until wait date |
| `recurring` | Template for recurring tasks |

### State Transitions

```
                ┌──────────────┐
                │   (create)   │
                └──────┬───────┘
                       │
                       ▼
    ┌───────────────────────────────────────┐
    │                 pending               │
    └────┬─────────────┬────────────┬───────┘
         │             │               │
   [complete]      [delete]        [wait]
         │             │               │
         ▼             ▼               ▼
    ┌─────────┐   ┌─────────┐    ┌─────────┐
    │completed│   │ deleted │    │ waiting │
    └─────────┘   └─────────┘    └────┬────┘
                                      │
                                 [wait expires]
                                      │
                                      └──────► pending
```

---

## Priority

| Value | Meaning | Urgency Coefficient |
|-------|---------|---------------------|
| `H` | High | +6.0 |
| `M` | Medium | +3.9 |
| `L` | Low | +1.8 |
| (none) | No priority | 0.0 |

---

## Annotation

Timestamped notes attached to tasks:

```json
{
  "entry": "20240116T090000Z",
  "description": "Added comments on line 42"
}
```

---

## Virtual Tags

Virtual tags are computed properties, not stored in the task data.

| Tag | Condition |
|-----|-----------|
| `+PENDING` | status == pending |
| `+COMPLETED` | status == completed |
| `+DELETED` | status == deleted |
| `+WAITING` | status == waiting |
| `+RECURRING` | status == recurring |
| `+ACTIVE` | has start time, status == pending |
| `+SCHEDULED` | has scheduled date |
| `+UNTIL` | has until date |
| `+ANNOTATED` | has annotations |
| `+TAGGED` | has at least one tag |
| `+PARENT` | is a recurring parent |
| `+CHILD` | has parent UUID |
| `+BLOCKING` | blocks other tasks (via depends) |
| `+BLOCKED` | blocked by other tasks |
| `+OVERDUE` | due < now, status == pending |
| `+TODAY` | due date == today |
| `+TOMORROW` | due date == tomorrow |
| `+WEEK` | due within 7 days |
| `+MONTH` | due within 30 days |
| `+QUARTER` | due within 90 days |
| `+YEAR` | due within 365 days |
| `+READY` | pending, not blocked, not waiting, scheduled <= now |

---

## JSON Format (Import/Export)

Taskwarrior uses JSON for data interchange:

```json
{
  "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "entry": "20240115T120000Z",
  "status": "pending",
  "description": "Review pull request",
  "project": "work.code-review",
  "priority": "H",
  "due": "20240120T170000Z",
  "tags": ["urgent", "review"],
  "annotations": [
    {
      "entry": "20240116T090000Z",
      "description": "Added comments on line 42"
    }
  ],
  "modified": "20240116T090000Z"
}
```

### Date Format

Taskwarrior uses ISO 8601 basic format (no separators): `%Y%m%dT%H%M%SZ`

Example: `20240115T120000Z`

---

## Hierarchical Projects

Projects use dot notation for hierarchy:

```
work
work.meetings
work.code-review
personal
personal.health
personal.health.exercise
```

Filtering `project:work` matches `work`, `work.meetings`, `work.code-review`.

---

## Urgency Calculation

```
urgency = Σ (coefficient × condition)

Default coefficients:
  priority.H      =  6.0
  priority.M      =  3.9
  priority.L      =  1.8
  project         =  1.0
  active          =  4.0
  scheduled       =  5.0
  age             =  2.0 (scaled 0.0-1.0 over 365 days)
  annotations     =  1.0
  tags            =  1.0
  due             = 12.0 (scaled by proximity)
  blocking        =  8.0
  blocked         = -5.0
```

### Due Urgency Curve

| Days until due | Factor |
|----------------|--------|
| < -14 (overdue) | 1.0 |
| -14 to 0 | 0.8 - 1.0 |
| 0 to 7 | 0.5 - 0.8 |
| 7 to 14 | 0.2 - 0.5 |
| > 14 | 0.0 - 0.2 |

---

## Rust Implementation Notes

```rust
// Core struct with serde for JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub uuid: Uuid,
    pub entry: DateTime<Utc>,
    pub status: Status,
    pub description: String,
    // Optional fields with #[serde(skip_serializing_if = "Option::is_none")]
    pub due: Option<DateTime<Utc>>,
    pub project: Option<String>,
    // ...
}

// Status as enum with strum for string conversion
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum Status { Pending, Completed, Deleted, Waiting, Recurring }

// Priority with urgency coefficient method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority { H, M, L }
```

