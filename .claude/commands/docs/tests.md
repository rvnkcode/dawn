---
description: Generate or update docs/tests.md with per-test Korean descriptions grounded in assertion bodies.
---

# Test Cases Documentation

Generate or update `docs/tests.md`. Collect every test case in the project and present them in per-module tables: test name plus a one-sentence Korean description.

**Language policy**: All instructions, workflow, and examples in this command file are in English. The **only** Korean text produced is the per-test description column inside `docs/tests.md`.

## Core Principles (MUST follow)

1. **Never infer behavior from a test function name.** The name is a hint; the **assertion** is the ground truth.
2. **Read the test body in full before writing its description.** Writing a description without reading the body (assertions, expected values, `matches!`, `unwrap()` targets) is forbidden.
3. **Capture the test's intent, not every asserted value.** Read the body in full to understand what the test is trying to verify, then write a single Korean sentence describing that intent. Concrete values belong in the description only when the intent itself IS that specific value (boundary cases like `zero_delta → "0s"`). "Handles", "processes", "manages" are still banned — state the intent directly.
4. **Verify AND/OR/containment semantics from the actual query or the final asserted result.** A name containing "and" may still be OR in the SQL.

## Workflow

### 1. Scope the work (incremental by default)

1. Find the last commit that touched `docs/tests.md`:
   `git log -1 --format=%H -- docs/tests.md`
2. If found, list Rust test files that changed since then:
   `git diff <commit>..HEAD --name-only -- '**/*.rs' 'tests/**/*.rs'`
   Filter to files that contain `#[test]` / `#[tokio::test]` / `#[rstest]`.
3. If the set is empty → report "no test changes since last doc update" and exit without reading files.
4. Otherwise, **the set of changed files is the scope**. Do not touch rows for unchanged files.

Full scan (`--full` argument, or when `docs/tests.md` does not yet exist) → enumerate every file with test functions via Grep and process all of them.

### 2. Read each in-scope file in FULL

- Do **not** stop at a Grep of function names. Each in-scope file must be opened with `Read`.
- ≤5 files in scope → read directly in the main context (no subagent).
- \>5 files in scope → delegate to **one** `general-purpose` subagent with all files listed. Do not spawn multiple subagents. The delegation prompt MUST repeat the "Description Rules" below verbatim.

### 3. Description rules (per test)

For every test function:

1. Read its body.
2. Locate the assertion(s): `assert_eq!`, `assert!(... .is_ok())`, `assert!(... .is_err())`, `matches!(...)`, `.unwrap()` followed by a comparison, etc.
3. Translate the **expected state/value** of that assertion into a single Korean sentence.
4. Self-check:
   - Ignore the test name. Does the description still match the test's intent?
   - If the description says "실패한다" / "성공한다", does it match `is_err()` / `is_ok()` in the body?
   - Are AND/OR/containment semantics confirmed against the query or the final asserted collection?

### 4. Known anti-patterns (observed failures)

Use these as reminders of why reading the body matters:

- `from_str_whitespace` — name suggests "whitespace fails", but the body is `"  5  ".parse::<Index>().unwrap() == 5` → whitespace is **trimmed and parsed**.
- `zero_delta` — name suggests a placeholder like `"-"`, but the body asserts `"0s"`.
- `list_tasks_filter_uid_and_index` — name suggests AND, but the SQL is `(t.id IN (?) OR tpr.row_id IN (?))` → **OR**.

In each case, reading the body prevents the mistake. Never skip step 3.1.

### 5. Output format (`docs/tests.md`)

- No top-level H1 (markdownlint MD025). The frontmatter `title` acts as the title.
- Group by module under H2 / H3. One table per file.
- Section order: Domain → Outbound → CLI (unit under `cli/src`) → E2E (`cli/tests`).
- Only the description column is written in Korean; everything else (headings, paths, test names) stays in English / code.

Template:

```markdown
---
title: Test Cases
---

## Domain Layer (`lib/src/domain`)

### `task/unique_id.rs`

| Test | 설명 |
| --- | --- |
| `test_name` | (한 문장 한국어 설명; 기대값/결과를 포함할 것) |
```

### 6. Update mode

If `docs/tests.md` already exists:

- Keep correct descriptions as-is.
- Rewrite only descriptions that **disagree** with the current test body.
  - "Disagreement" = wrong intent: fails vs succeeds, AND vs OR, rejects vs accepts, placeholder vs concrete value when the concrete value IS the intent.
  - Under-claim (description omits a secondary assertion but the intent is correctly stated) is NOT a disagreement — leave it.
- Add rows for newly introduced tests.
- Remove rows for tests that no longer exist.

### 7. Verification (required before finishing)

1. Run `markdownlint` expectations mentally — in particular, avoid MD025 (multiple H1).
2. Report to the user a summary of: rows added / rewritten / removed, and any tests whose intent is ambiguous enough to warrant a second look.
3. **Full scan only**: additionally pick 3–5 tests at random, re-read their bodies, and diff against the written descriptions. Skip in incremental mode — the scoped files were already read in full.

## Arguments (optional)

- No argument → incremental mode (diff since last `docs/tests.md` commit).
- `--full` → full scan of every test file (use when intentionally rewriting or when incremental can't be trusted, e.g. after a large refactor).
- Path argument (e.g. `lib/src/domain`) → only refresh rows whose test files live under that path. Takes precedence over incremental scoping.
