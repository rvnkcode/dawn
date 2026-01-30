# Taskwarrior Filter System

Filters select tasks based on various criteria. Used in reports, modifications, and deletions.

```
task <filter> <command> [<modifications>]
```

---

## ID Filters

| Syntax | Example | Description |
|--------|---------|-------------|
| `<id>` | `5` | Single task by ID |
| `<id>-<id>` | `5-10` | Range of IDs |
| `<id>,<id>` | `5,7,9` | Multiple IDs |
| `<uuid>` | `a1b2c3d4` | Partial or full UUID |

---

## Tag Filters

| Syntax | Description |
|--------|-------------|
| `+tag` | Tasks with tag |
| `-tag` | Tasks without tag |
| `+VIRTUAL` | Virtual tags (uppercase) |

---

## Attribute Filters

| Syntax | Example | Description |
|--------|---------|-------------|
| `attr:value` | `project:work` | Exact match |
| `attr.mod:value` | `due.before:tomorrow` | With modifier |
| `attr:` | `project:` | Attribute is empty |

---

## Attribute Modifiers

### String Modifiers

| Modifier | Description |
|----------|-------------|
| `is` | Exact match (default) |
| `isnt` | Not exact match |
| `has` | Contains substring |
| `hasnt` | Not contains |
| `startswith` | Starts with |
| `endswith` | Ends with |
| `word` | Contains word |
| `noword` | Not contains word |

### Date Modifiers

| Modifier | Description | Example |
|----------|-------------|---------|
| `before` | Earlier than | `due.before:tomorrow` |
| `after` | Later than | `due.after:today` |
| `by` | On or before | `due.by:eow` |
| `is` | Exact date | `due.is:2024-01-20` |
| `isnt` | Not exact date | `due.isnt:today` |

### Numeric Modifiers

| Modifier | Description |
|----------|-------------|
| `gt` | Greater than |
| `gte` | Greater or equal |
| `lt` | Less than |
| `lte` | Less or equal |

---

## Date Expressions

### Named Dates

| Expression | Description |
|------------|-------------|
| `now` | Current timestamp |
| `today` | Start of today |
| `yesterday` | Start of yesterday |
| `tomorrow` | Start of tomorrow |
| `monday`...`sunday` | Next occurrence |

### End/Start of Period

| Expression | Description |
|------------|-------------|
| `eod` / `sod` | End/Start of day |
| `eow` / `sow` | End/Start of week |
| `eoww` / `soww` | End/Start of work week |
| `eom` / `som` | End/Start of month |
| `eoq` / `soq` | End/Start of quarter |
| `eoy` / `soy` | End/Start of year |

### Relative Dates

| Format | Example | Description |
|--------|---------|-------------|
| `+Nd` | `+3d` | 3 days from now |
| `-Nd` | `-1d` | 1 day ago |
| `+Nw` | `+2w` | 2 weeks |
| `+Nm` | `+1m` | 1 month |
| `+Nq` | `+1q` | 1 quarter |
| `+Ny` | `+1y` | 1 year |

### Special

| Expression | Description |
|------------|-------------|
| `later` | Far future (9999-12-30) |
| `someday` | Same as later |

---

## Logical Operators

| Operator | Description | Precedence |
|----------|-------------|------------|
| `(`, `)` | Grouping | Highest |
| `!`, `not` | Negation | High |
| `and` | Conjunction | Medium |
| `or` | Disjunction | Low |
| `xor` | Exclusive or | Low |

### Implicit AND

Consecutive filters without operators are joined with AND:

```
task project:work +urgent status:pending
# Equivalent to:
task project:work and +urgent and status:pending
```

---

## Examples

```bash
# Tasks due today
task due:today list

# High priority work tasks
task project:work priority:H list

# Overdue tasks not in project "home"
task +OVERDUE project.isnt:home list

# Tasks due within a week, excluding waiting
task due.before:+7d -WAITING list

# Complex filter with grouping
task '(project:work or project:study) and +urgent' list

# Tasks with "bug" in description
task description.has:bug list

# Tasks due between dates
task due.after:2024-01-01 due.before:2024-02-01 list
```

---

## Filter Grammar (Simplified)

```
filter     := or_expr
or_expr    := xor_expr ('or' xor_expr)*
xor_expr   := and_expr ('xor' and_expr)*
and_expr   := not_expr ('and'? not_expr)*
not_expr   := 'not'? primary
primary    := '(' or_expr ')' | tag | attribute | id

tag        := ('+' | '-') IDENTIFIER
attribute  := IDENTIFIER ('.' MODIFIER)? ':' VALUE?
id         := NUMBER | NUMBER '-' NUMBER | UUID
```

---

## Rust Implementation Notes

```rust
// Filter AST
pub enum Filter {
    Id(IdFilter),
    Tag { name: String, positive: bool },
    Attribute { name: String, modifier: Option<Modifier>, value: String },
    And(Box<Filter>, Box<Filter>),
    Or(Box<Filter>, Box<Filter>),
    Not(Box<Filter>),
}

// Parsing approach options:
// 1. nom - parser combinators
// 2. pest - PEG grammar
// 3. Hand-written recursive descent

// Evaluation
impl Filter {
    pub fn matches(&self, task: &Task, ctx: &FilterContext) -> bool;
}
```
