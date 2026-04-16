# Taskwarrior Filter System

Filters select tasks based on various criteria. Used in reports, modifications, and deletions.

```sh
task <filter> <command> [<modifications>]
```

> **How this is parsed:** See [parsing-pipeline.md](parsing-pipeline.md) for the lexer → categorize → desugar → evaluate pipeline. This document focuses on surface syntax and semantics.

## ID Filters

| Syntax      | Example    | Lexer Type | Description           |
| ----------- | ---------- | ---------- | --------------------- |
| `<id>`      | `5`        | `number`   | Single task by ID     |
| `<id>-<id>` | `5-10`     | `set`      | Range of IDs          |
| `<id>,<id>` | `5,7,9`    | `set`      | Multiple IDs          |
| mixed       | `1,3,5-10` | `set`      | Ranges + singles      |
| `<uuid>`    | `a1b2c3d4` | `uuid`     | Partial or full UUID  |

**Note:** `set` token requires count ≥ 2. A bare `5` is `number`, not `set`. This difference drives default command resolution — see `commands.md`.

## Tag Filters

| Syntax     | Description               |
| ---------- | ------------------------- |
| `+tag`     | Tasks with tag            |
| `-tag`     | Tasks without tag         |
| `+VIRTUAL` | Virtual tags (uppercase)  |

## Attribute Filters

| Syntax           | Example              | Description        |
| ---------------- | -------------------- | ------------------ |
| `attr:value`     | `project:work`       | Exact match        |
| `attr.mod:value` | `due.before:tomorrow`| With modifier      |
| `attr:`          | `project:`           | Attribute is empty |

## Attribute Modifiers

### String Modifiers

| Modifier     | Description           |
| ------------ | --------------------- |
| `is`         | Exact match (default) |
| `isnt`       | Not exact match       |
| `has`        | Contains substring    |
| `hasnt`      | Not contains          |
| `startswith` | Starts with           |
| `endswith`   | Ends with             |
| `word`       | Contains word         |
| `noword`     | Not contains word     |

### Date Modifiers

| Modifier | Description    | Example              |
| -------- | -------------- | -------------------- |
| `before` | Earlier than   | `due.before:tomorrow`|
| `after`  | Later than     | `due.after:today`    |
| `by`     | On or before   | `due.by:eow`         |
| `is`     | Exact date     | `due.is:2024-01-20`  |
| `isnt`   | Not exact date | `due.isnt:today`     |

### Numeric Modifiers

| Modifier | Description      |
| -------- | ---------------- |
| `gt`     | Greater than     |
| `gte`    | Greater or equal |
| `lt`     | Less than        |
| `lte`    | Less or equal    |

## Date Expressions

### Named Dates

| Expression          | Description          |
| ------------------- | -------------------- |
| `now`               | Current timestamp    |
| `today`             | Start of today       |
| `yesterday`         | Start of yesterday   |
| `tomorrow`          | Start of tomorrow    |
| `monday`...`sunday` | Next occurrence      |

### End/Start of Period

| Expression      | Description           |
| --------------- | --------------------- |
| `eod` / `sod`   | End/Start of day      |
| `eow` / `sow`   | End/Start of week     |
| `eoww` / `soww` | End/Start of work week|
| `eom` / `som`   | End/Start of month    |
| `eoq` / `soq`   | End/Start of quarter  |
| `eoy` / `soy`   | End/Start of year     |

### Relative Dates

| Format | Example | Description      |
| ------ | ------- | ---------------- |
| `+Nd`  | `+3d`   | 3 days from now  |
| `-Nd`  | `-1d`   | 1 day ago        |
| `+Nw`  | `+2w`   | 2 weeks          |
| `+Nm`  | `+1m`   | 1 month          |
| `+Nq`  | `+1q`   | 1 quarter        |
| `+Ny`  | `+1y`   | 1 year           |

### Special

| Expression | Description            |
| ---------- | ---------------------- |
| `later`    | Far future (9999-12-30)|
| `someday`  | Same as later          |

---

## Operator Precedence

Full operator table from `src/Eval.cpp:44-86`. Higher number = higher precedence. All are left-associative unless noted.

| Prec | Operator              | Kind    | Description                |
| ---- | --------------------- | ------- | -------------------------- |
| 16   | `^`                   | binary  | Exponent (right-assoc)     |
| 15   | `!`                   | unary   | Logical NOT (right-assoc)  |
| 15   | `_neg_` `_pos_`       | unary   | Unary minus / plus         |
| 14   | `_hastag_` `_notag_`  | binary  | Tag presence (sugar)       |
| 13   | `*` `/` `%`           | binary  | Multiply / divide / mod    |
| 12   | `+` `-`               | binary  | Add / subtract             |
| 10   | `<` `<=` `>` `>=`     | binary  | Relational                 |
| 9    | `=` `==` `!=` `!==`   | binary  | Equality                   |
| 8    | `~` `!~`              | binary  | Regex match / non-match    |
| 5    | `and`                 | binary  | Logical AND                |
| 4    | `or`                  | binary  | Logical OR                 |
| 3    | `xor`                 | binary  | Logical XOR                |
| 0    | `(` `)`               | group   | Grouping                   |

### Implicit AND

Consecutive filters without operators are joined with AND:

```sh
task project:work +urgent status:pending
# Equivalent to:
task project:work and +urgent and status:pending
```

## Examples

```sh
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

## Filter Grammar (Simplified)

```txt
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

## Desugaring

Surface syntax is rewritten into canonical expressions before evaluation. See `parsing-pipeline.md` stage 3 for the full flow.

### Tag desugaring

```txt
+urgent   →  tags _hastag_ 'urgent'
-waiting  →  tags _notag_ 'waiting'
```

Code: `src/CLI2.cpp` `desugarFilterTags()` (1041).

### Attribute desugaring (modifier → operator)

Attribute pairs `name[.modifier]:value` expand via this mapping:

| Modifier                        | Operator | Example → canonical form            |
| ------------------------------- | -------- | ----------------------------------- |
| (none)                          | `=`      | `project:work` → `project = 'work'` |
| `is`, `equals`                  | `==`     | `priority.is:H` → `priority == 'H'` |
| `not`                           | `!=`     | `status.not:pending`                |
| `isnt`                          | `!==`    | `project.isnt:work`                 |
| `before`, `under`, `below`      | `<`      | `due.before:tomorrow`               |
| `after`, `over`, `above`        | `>`      | `due.after:today`                   |
| `by`                            | `<=`     | `due.by:eow`                        |
| `has`, `contains`               | `~`      | `description.has:bug`               |
| `hasnt`                         | `!~`     | `description.hasnt:bug`             |
| `startswith`, `left`            | `~`      | rewrites value to `^value`          |
| `endswith`, `right`             | `~`      | rewrites value to `value$`          |
| `word`                          | `~`      | word-boundary regex                 |
| `noword`                        | `!~`     | negated word-boundary regex         |
| `none`                          | `==`     | empty check: `project.none:`        |
| `any`                           | `!==`    | non-empty check: `project.any:`     |

Code: `src/CLI2.cpp` `desugarFilterAttributes()` (1095-1254).

### Pattern and plain word

```txt
/bug/              →  description ~ 'bug'
shopping (plain)   →  description ~ 'shopping'
```

Plain-word detection requires the surrounding tokens to be non-operators (or `(`, `)`, `and`, `or`, `xor`).

Code: `src/CLI2.cpp` `desugarFilterPatterns()` (1257), `desugarFilterPlainArgs()` (1622).

### ID / UUID expansion

All collected `number`, `set`, and `uuid` tokens in FILTER position are merged into a single OR expression:

```txt
1,2-3 uuid1 uuid2
↓
( (id == 1)
  or ((id >= 2) and (id <= 3))
  or (uuid = 'uuid1')
  or (uuid = 'uuid2') )
```

- Descending ranges (`10-5`) are automatically swapped to ascending.
- Inserted at the position of the first ID/UUID token.

Code: `src/CLI2.cpp` `insertIDExpr()` (1443-1578).

### Implicit AND

Adjacent FILTER tokens without an explicit operator are joined with `and`:

```sh
project:work +urgent status:pending
# becomes
project = 'work' and tags _hastag_ 'urgent' and status = 'pending'
```

Insertion rule: between `)`/non-op and `(`/non-op (not between operators or empty parens).

Code: `src/CLI2.cpp` `insertJunctions()` (1741-1780).
