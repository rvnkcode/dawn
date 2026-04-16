# Taskwarrior CLI Parsing Pipeline

How Taskwarrior transforms raw CLI argv into executable filter + command + modifications. This is a multi-stage pipeline — fundamentally different from a single-pass parser like `clap`.

## Pipeline Stages

```txt
argv  →  [1] Lex  →  [2] Categorize  →  [3] Desugar  →  [4] Evaluate
          tokens     role tags         canonical       tree walk
                                        expression
```

| Stage | Input | Output | Source |
| --- | --- | --- | --- |
| 1. Lex | Raw strings | Typed tokens (17 types) | `src/Lexer.cpp` |
| 2. Categorize | Tokens | Tokens + tags (FILTER/CMD/...) | `src/CLI2.cpp` (`categorizeArgs`) |
| 3. Desugar | Tagged tokens | Canonical filter expression | `src/CLI2.cpp` (`desugar*`) |
| 4. Evaluate | Expression | Bool per task | `src/Eval.cpp` |

## Stage 1: Lexer Token Types

**File:** `src/Lexer.h` (enum `Type`), `src/Lexer.cpp` (`token()`)

Tokens are tried in precedence order; first match wins.

| Precedence | Type | Example | Notes |
| --- | --- | --- | --- |
| 1 (highest) | `string` | `"hello world"`, `'x'` | Supports `\n`, `\t`, unicode escapes |
| 2 | `date` | `2024-01-20`, `eow`, `today` | Parsed via `Datetime` |
| 3 | `duration` | `3d`, `2w`, `1h` | Parsed via `Duration` |
| 4 | `url` | `https://example.com` | Case-insensitive `http[s]://` |
| 5 | `pair` | `project:work`, `due.before:x` | `name[.modifier]<sep>value`, sep: `:` `=` `::` `:=` |
| 6 | **`set`** | `1,2,3`, `5-10`, `1,3,5-10` | **Requires count ≥ 2** |
| 7 | `dom` | `tags.urgent`, `rc.color` | Domain Object Model reference |
| 8 | `hex` | `0xFF` | `0x` + 1+ hex digits |
| 9 | **`number`** | `1`, `42`, `3.14` | No leading zeros (except `0`) |
| 10 | `separator` | `--` | Terminator; all following tokens become `word` |
| 11 | `tag` | `+urgent`, `-waiting` | Must be preceded by start/space/`(`/`)` |
| 12 | `path` | `/usr/local/bin` | ≥ 4 slashes |
| 13 | `substitution` | `/old/new/g` | sed-style |
| 14 | `pattern` | `/regex/` | Regex |
| 15 | `op` | `and`, `or`, `==`, `!~`, `+`, `(` | See operator precedence in `filter-system.md` |
| 16 | `identifier` | `project`, `mytag` | Starts non-digit, no `:`/`=`/space |
| 17 (lowest) | `word` | catch-all | Always succeeds |

### The `set` vs `number` Distinction (CRITICAL)

```txt
"1"      → number   (count = 1)
"1,2"    → set      (count = 2)
"5-10"   → set      (count = 2: [5, 10])
"1,5-10" → set      (count = 3)
```

**`isSet()` requires `count > 1`** (`src/Lexer.cpp:697`). A single number becomes `number`, never `set`. This distinction drives default command resolution (see `commands.md`).

### UUID Partial Matching

- Full format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (36 chars)
- **Minimum: 8 hex chars** (`uuid_min_length = 8`, `src/Lexer.cpp:40`)
- Greedy: consumes up to 36 chars while respecting the hex/hyphen pattern
- Boundary: must be followed by EOS, whitespace, or single-char operator

Examples: `a1b2c3d4` (8), `a1b2c3d4-e5f6` (13), full 36-char UUID — all valid.

## Stage 2: Argument Categorization

**File:** `src/CLI2.cpp` — `analyze()` (line 453), `categorizeArgs()` (line 827), `findCommand()` (line 986)

Each argument accumulates tags. Major tags:

| Tag | Meaning |
| --- | --- |
| `BINARY` | argv[0] |
| `CMD` | The command keyword |
| `FILTER` | Part of filter expression |
| `MODIFICATION` | Task modification (for add/modify/done) |
| `MISCELLANEOUS` | Passed to command (reports, externals) |
| `RC` | `rc.<name>=<value>` override |
| `CONFIG` | `rc.<name>:<value>` override (with modifier) |
| `TERMINATED` | After `--` |
| `ORIGINAL` | From original CLI (not injected) |
| `ASSUMED` | Injected by `defaultCommand()` |
| `DEFAULT` | Injected from `rc.default.command` |

### Pipeline Steps (inside `analyze()`)

1. `handleArg0()` — capture binary name, handle symlinks (e.g., `cal` → inject `calendar`)
2. `lexArguments()` — tokenize; apply `--` terminator semantics
3. `aliasExpansion()` — recursively expand aliases (limit: 10 iterations)
4. `findCommand()` — identify the command argument, tag it `CMD`
5. `demotion()` — special case: `add`/`log` demote `-tag` to plain word
6. `canonicalizeNames()` — expand abbreviated attribute names (e.g., `pro` → `project`)
7. `categorizeArgs()` — assign FILTER/MODIFICATION/MISCELLANEOUS based on command capabilities and position
8. `parenthesizeOriginalFilter()` — wrap user-supplied filter in `( ... )`

### Categorization Rules

Based on command capability flags (see `commands.md` for the full table):

| accepts_filter | accepts_modifications | Before CMD | After CMD |
| --- | --- | --- | --- |
| yes | yes | FILTER | MODIFICATION |
| yes | no | FILTER | FILTER |
| no | yes | MODIFICATION | MODIFICATION |

### Terminator Semantics

After `--`, all tokens are retyped to `word` regardless of lexical match.

```txt
task add -- -tag           # "-tag" is description, not a tag filter
task add -- due:tomorrow   # "due:tomorrow" is part of description
```

### Pair Decomposition

`pair` tokens are split into `name`, `modifier`, `separator`, `value`:

| Raw | name | modifier | separator | value |
| --- | --- | --- | --- | --- |
| `project:work` | project | (none) | `:` | work |
| `due.before:tomorrow` | due | before | `:` | tomorrow |
| `rc.color:on` | rc | color | `:` | on (CONFIG tag) |
| `project=work` | project | (none) | `=` | work |

## Stage 3: Desugar

**File:** `src/CLI2.cpp` — `desugarFilterTags()` (1041), `desugarFilterAttributes()` (1095), `desugarFilterPatterns()` (1257), `desugarFilterPlainArgs()` (1622), `insertIDExpr()` (1443)

Transforms surface syntax to canonical filter expressions. See `filter-system.md` for full desugaring tables.

Quick summary:

```txt
+urgent                     →  tags _hastag_ 'urgent'
-waiting                    →  tags _notag_ 'waiting'
project:work                →  project = 'work'
due.before:tomorrow         →  due < tomorrow
/bug/                       →  description ~ 'bug'
meeting (plain word)        →  description ~ 'meeting'
1,2,5-7                     →  (id==1) or (id==2) or ((id>=5) and (id<=7))
```

Implicit `and` is inserted between consecutive FILTER tokens without explicit operators.

## Stage 4: Evaluate

**File:** `src/Eval.cpp`

- Infix → postfix (Shunting Yard variant)
- Operator precedence: see `filter-system.md`
- Evaluates per task, returns bool

## Worked Examples

### `task 1 2 done`

```txt
Lex:        number(1)  number(2)  word(done)
Command:    "done" ← CMD
Categorize: 1→FILTER  2→FILTER  done→CMD
Desugar:    ((id == 1) or (id == 2))
```

Effect: mark tasks 1 and 2 as done.

### `task 1,2,3 list`

```txt
Lex:        set(1,2,3)  word(list)
Command:    "list" ← CMD
Categorize: 1,2,3→FILTER  list→CMD
Desugar:    ((id == 1) or (id == 2) or (id == 3))
```

Effect: list tasks 1, 2, 3.

### `task 1` (no command)

```txt
Lex:        number(1)
Command:    (none found)
Default:    number token present → inject "information"
Categorize: 1→FILTER  information→CMD
Desugar:    (id == 1)
```

Effect: show detailed info of task 1.

### `task 1,2,3` (no command)

```txt
Lex:        set(1,2,3)
Command:    (none found)
Default:    NO number/uuid token → inject rc.default.command (e.g., "next")
Categorize: 1,2,3→FILTER  next→CMD
Desugar:    ((id == 1) or (id == 2) or (id == 3))
```

Effect: list via default report. See `commands.md` for the full default-command logic.

### `task project:work +urgent modify due:tomorrow`

```txt
Lex:        pair(project:work)  tag(+urgent)  word(modify)  pair(due:tomorrow)
Command:    "modify" ← CMD (accepts filter AND modifications)
Categorize: project:work→FILTER  +urgent→FILTER  modify→CMD  due:tomorrow→MODIFICATION
Desugar:    Filter: project='work' and tags _hastag_ 'urgent'
            Mod:    set due = tomorrow
```

### `task (project:a or project:b) and +next list`

```txt
Lex:        op(()  pair(project:a)  op(or)  pair(project:b)  op())
            op(and)  tag(+next)  word(list)
Command:    "list" ← CMD
Categorize: all before "list" → FILTER
Desugar:    (project='a' or project='b') and tags _hastag_ 'next'
```

## Comparison with clap

| Aspect | clap (Dawn today) | Taskwarrior |
| --- | --- | --- |
| Architecture | Single-pass declarative | Multi-stage pipeline |
| Tokenization | argv as strings | Domain-aware lexer (17 types) |
| Role assignment | Type-driven (subcommand) | Tag-driven, command-capability-aware |
| Filter grammar | None | Full expression grammar |
| Syntactic sugar | None | Tags, attributes, patterns, plain words |
| Default command | Subcommand required | `information` or `rc.default.command` |
| Terminator (`--`) | Supported by clap | Full retyping of following tokens |

Dawn can incrementally adopt pieces — see plan files for scope decisions.

## Key Source References

| Concern | File |
| --- | --- |
| Token type enum | `src/Lexer.h` (45-64) |
| Token pattern matching | `src/Lexer.cpp` (52-78, 455-706) |
| UUID rules | `src/Lexer.cpp` (39-40, 455-477) |
| Set rules (count > 1) | `src/Lexer.cpp` (673-706) |
| Argument tags | `src/CLI2.h` (class `A2`) |
| Pipeline entry | `src/CLI2.cpp` `analyze()` (453) |
| Categorization | `src/CLI2.cpp` (827-920) |
| Default command injection | `src/CLI2.cpp` (1791-1850) |
| Tag desugaring | `src/CLI2.cpp` (1041-1071) |
| Attribute desugaring | `src/CLI2.cpp` (1095-1254) |
| ID/UUID expansion | `src/CLI2.cpp` (1443-1578) |
| Implicit AND insertion | `src/CLI2.cpp` (1741-1780) |
| Operator precedence | `src/Eval.cpp` (44-86) |
