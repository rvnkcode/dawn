# Taskwarrior Confirmation & Prompt Behavior

> Source: `src/commands/Command.cpp` (`permission()`), `src/util.cpp` (`confirm4`), `src/Filter.cpp` (`safety()`), each `src/commands/Cmd*.cpp`.

This document is the single source of truth for **what prompts Taskwarrior shows, when, and what each user response does**. The default config values used below are `confirmation=on`, `bulk=3`, `recurrence.confirmation=prompt`, `allow.empty.filter=true`.

## 1. Confirmation primitives

### 1.1 `Command::permission(question, quantity)` — Command.cpp:315–363

Logic:

```cpp
if (_read_only || _permission_all) return true;
if (_permission_quit)              return false;

if (quantity == 1) {
  if (!_needs_confirm || !confirmation) return true;     // no prompt
  return confirm(question);                              // (yes/no)
}

// quantity > 1
if ((bulk == 0 || quantity < bulk) && (!_needs_confirm || !confirmation))
  return true;                                            // no prompt

return confirm4(question);                                // (yes/no/all/quit)
```

Effective threshold table (`bulk=3` default; "y/n/a/q" = `(yes/no/all/quit)`):

| `_needs_confirm`         | `confirmation` | qty 1      | qty 2          | qty ≥ `bulk`     |
| ------------------------ | -------------- | ---------- | -------------- | ---------------- |
| **true** (delete, purge) | on (default)   | `(yes/no)` | **y/n/a/q**    | y/n/a/q          |
| **true**                 | off            | auto-yes   | auto-yes       | y/n/a/q (forced) |
| **false** (others)       | on             | auto-yes   | auto-yes       | y/n/a/q          |
| **false**                | off            | auto-yes   | auto-yes       | y/n/a/q (forced) |

**Critical insight:** the "bulk threshold" only applies to `_needs_confirm=false` commands or when `confirmation=off`. For `delete` / `purge` under default config, the (y/n/a/q) prompt kicks in at **quantity ≥ 2**, not at `bulk`.

`bulk=0` means "infinite bulk" — never force-prompt.

### 1.2 `confirm4(question)` — util.cpp:85–127

Format: `{question} (yes/no/all/quit)` — trailing space, no newline.

Input is `lowerCase(trim(stdin))` then prefix-autocompleted against `["Yes", "yes", "no", "All", "all", "quit"]` until exactly one match.

| Response            | This call | After this call                              |
| ------------------- | --------- | -------------------------------------------- |
| `y`/`yes`/`Y`/`Yes` | yes       | each subsequent call re-prompts              |
| `n`/`no`            | no        | each subsequent call re-prompts              |
| `a`/`all`/`A`/`All` | yes       | sets `_permission_all`; rest auto-yes        |
| `q`/`quit`          | no        | sets `_permission_quit`; rest auto-no        |
| EOF (Ctrl-D)        | no        | same as `quit`                               |
| ambiguous / empty   | -         | re-prompts indefinitely                      |
| SIGINT (Ctrl-C)     | -         | prints "Interrupted: No changes made."; exit |

`verbose("blank")` adds a blank line between prompts (skipped on first iteration).

### 1.3 `confirm(question)` — libshared (simple yes/no)

Format: `{question} (yes/no)`. Returns `bool`. Same autocomplete rules.

Used standalone (not via `permission()`):

- **`Filter::safety`**
  - Q: `This command has no filter, and will modify all (including completed and deleted) tasks.  Are you sure?`
  - On no: throw `"Command prevented from running."` → rc=2
- **`Context::createDefaultConfig`**
  - Q: `A configuration file could not be found in {dir}\n\nWould you like a sample {file} created, so Taskwarrior can proceed?`
  - On no: throw `"Cannot proceed without rc file."` → rc=2
- **`CmdDelete` recurrence siblings/children**
  - Q: `This is a recurring task.  Do you want to delete all pending recurrences of this same task?`
  - On no: skip sibling/child deletion (parent task still deleted)
- **`CmdPurge::handleChildren`**
  - Q: `Task '{desc}' is a recurrence template. All its {N} deleted children tasks will be purged as well. Continue?`
  - On no: throw `"Purge operation aborted."` → rc=2
- **`CmdEdit`**
  - Q: `Do you wish to manually edit {N} tasks?`
  - On no: return rc=2
- **`CmdSync`** (init only)
  - Q: `Please confirm that you wish to upload all your tasks to the Taskserver`
  - On no: throw → rc=2
- **`TDB2::revert`** (undo)
  - Q: `The undo command is not reversible.  Are you sure you want to revert to the previous state?`
  - On no: skip revert, no error
- **`dependencyChainOnComplete`** etc.
  - Q: `Would you like the dependency chain fixed?`
  - On no: skip fix, no error

## 2. Per-command prompt behavior

### 2.1 Modifying commands using `permission()`

All use `format("{Verb} task {ID} '{description}'?")` as the question. `{ID}` is `Task::identifier(true)` — numeric `id` if present, else first 8 chars of UUID.

| Command     | needs_confirm | Question                                 | Diff prefix\* |
| ----------- | ------------- | ---------------------------------------- | ------------- |
| `delete`    | **true**      | `Delete task {ID} '{desc}'?`             | no            |
| `purge`     | **true**      | `Permanently remove task {ID} '{desc}'?` | no            |
| `done`      | false         | `Complete task {ID} '{desc}'?`           | yes           |
| `modify`    | false         | `Modify task {ID} '{desc}'?`             | yes           |
| `start`     | false         | `Start task {ID} '{desc}'?`              | yes           |
| `stop`      | false         | `Stop task {ID} '{desc}'?`               | yes           |
| `annotate`  | false         | `Annotate task {ID} '{desc}'?`           | yes           |
| `denotate`  | false         | `Denotate task {ID} '{desc}'?`           | yes           |
| `append`    | false         | `Append to task {ID} '{desc}'?`          | yes           |
| `prepend`   | false         | `Prepend to task {ID} '{desc}'?`         | yes           |
| `duplicate` | false         | `Duplicate task {ID} '{desc}'?`          | no            |

\* "Diff prefix" = whether `before.diff(task)` is prepended to the question (lines like `- Project will be set to 'work'.` listing every attribute change). `delete`, `purge`, `duplicate` show no diff.

**On `no` per task**: every command except `purge` prints `Task not <verb>.` (e.g. `Task not deleted.`, `Task not completed.`), sets rc=1 for that task, and `break`s the loop on `quit`. **Purge is the only outlier**: silent skip on no/quit, no rc change, loop continues even after `quit` (`_permission_quit` simply auto-no's the rest). This is because purge already gates by `status == deleted`; non-purge skips are not user-visible failures.

### 2.2 Self-managed prompts (do not use `permission()`)

- **`edit`**
  - Trigger: `filtered.size() > bulk` (and `bulk != 0`)
  - Q: `Do you wish to manually edit {N} tasks?`
  - On no: `return 2` (rc=2)
- **`undo`**
  - Trigger: `confirmation=on` (always when invoked)
  - Q: `The undo command is not reversible.  Are you sure you want to revert to the previous state?`
  - On no: revert skipped, rc=0
- **`sync init`**
  - Trigger: `initialize` keyword + `confirmation=on`
  - Q: `Please confirm that you wish to upload all your tasks to the Taskserver`
  - On no: throw → rc=2

### 2.3 No prompt

`add`, `log`, `import`, regular `sync`, all report commands (`list`, `next`, `all`, etc.), all metadata commands.

## 3. Recurrence sub-prompts (within delete / purge)

Both delete and purge handle recurrence parent/children, but with **different policies**:

### 3.1 `delete` — when deleting a recurring instance (CmdDelete.cpp:107–162)

If the deleted task has `parent` (i.e., it's a child instance) **and** `recurrence.confirmation` triggers `yes`:

```text
This is a recurring task.  Do you want to delete all pending recurrences of this same task?
```

| `recurrence.confirmation` | Behavior                                                                            |
| ------------------------- | ----------------------------------------------------------------------------------- |
| `yes`/`true`/`1`/`on`     | auto-delete all siblings + parent (no prompt)                                       |
| `prompt` (default)        | prompt `(yes/no)`. yes → delete siblings + parent. no → only original task deleted  |
| other                     | no prompt, only original task deleted                                               |

If the deleted task has children (it's a recurrence parent), same prompt and policy applies to its children.

### 3.2 `purge` — when purging a recurrence parent (CmdPurge.cpp:85–129)

Same policy structure but **fail-closed**:

If any child has `status != deleted`:

```text
throw "Task '{desc}' is a recurrence template. Its child task {ID} must be deleted before it can be purged."
→ rc=2
```

If all children are deleted, prompt:

```text
Task '{desc}' is a recurrence template. All its {N} deleted children tasks will be purged as well. Continue?
```

- `yes`/`true`/`1`/`on`: auto-purge children (no prompt)
- `prompt` (default): prompt `(yes/no)`. yes -> purge children. no -> throw `Purge operation aborted.` rc=2
- other (`no`/`false`/anything else): throw `Purge operation aborted.` rc=2

> **Why `getBoolean(...) || (... == "prompt" && confirm(...))` ?** TW `getBoolean` recognizes only `yes/true/1/on` as true and treats `prompt` as false. So the 3-way is encoded by combining a boolean check with an explicit string compare for `"prompt"`.

## 4. Filter safety prompts (pre-filter)

`Filter::safety()` runs inside `filter.subset(filtered)` for any `_read_only=false` command with `_accepts_filter=true`.

| Condition                                                   | Behavior                                                                                                  |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| filter present                                              | pass                                                                                                      |
| no filter, `allow.empty.filter=false`                       | throw `You did not specify a filter, and with the 'allow.empty.filter' value, no action is taken.` → rc=2 |
| no filter, `allow.empty.filter=true`, `confirmation=off`    | throw `Command prevented from running.` → rc=2                                                            |
| no filter, `allow.empty.filter=true`, `confirmation=on`     | prompt `(yes/no)`; no → throw `Command prevented from running.` → rc=2                                    |

> Combined with `_uses_context=true`, an active context counts as a filter. So `task purge` with no explicit args but with an active context bypasses safety.

## 5. Exit codes — Context::run() catch block (Context.cpp:728–836)

- **0** — normal completion (including 0 actions when all skipped or no matches of the right status)
- **1** — `CmdPurge`/`CmdEdit` `return 1` for empty-filter footnote; modifying commands' per-task `no`/`quit`/wrong-status; `confirm4` SIGINT `exit(1)`
- **2** — `throw const std::string&` (Filter safety, recurrence aborts, sync init refusal, `CmdEdit` bulk refusal, etc.)
- **3** — `catch (...)` (unknown exception)
- **4** — `catch (int)` (hook integer throw)

## 6. Dawn parity guidance

When implementing TW-parity commands in Dawn:

1. **Pick prompt style by capability flags, not intent** — see [commands.md "Dawn Handler Dispatch"](commands.md#dawn-handler-dispatch). `delete` and `purge` look similar but their per-task on-no/on-quit semantics differ.
2. **Per-task flow template** — `for task in filtered: if eligible(task): if permission(...): action(task) else: handle_skip(task)`.
3. **Counter semantics** — count what was actually applied, not what was filtered. Recurrence-parent purge counts children too.
4. **Rc convention** — failure to action a single task in a multi-task batch returns rc=1 unless the command (purge) explicitly silences it.
5. **Transactional commit** — TW commits all in-memory changes only after `dispatch()` returns. Mid-loop exceptions roll back everything. Match this with a transaction or staging buffer.
6. **Undo coverage** — TW writes undo lines in `tdb2.update()` but not in `tdb2.purge()`. Purge is intentionally non-reversible.
