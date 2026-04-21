---
description: Generate or update docs/tests.md with per-test Korean descriptions grounded in assertion bodies.
---

# Test Cases Documentation

Generate or update `docs/tests.md`. Collect every test case in the project and present them in per-module tables: test name plus a one-sentence Korean description.

**Language policy**: All instructions, workflow, and examples in this command file are in English. The **only** Korean text produced is the per-test description column inside `docs/tests.md`.

## Core Principles (MUST follow)

1. **Never infer behavior from a test function name.** The name is a hint; the **assertion** is the ground truth.
2. **Read the test body in full before writing its description.** Writing a description without reading the body (assertions, expected values, `matches!`, `unwrap()` targets) is forbidden.
3. **Include the concrete expected value/outcome** in the description (e.g., returns `"0s"`, returns `Err(TooSmall)`, returns an empty `Vec`). Avoid vague verbs like "handles", "processes", "manages".
4. **Verify AND/OR/containment semantics from the actual query or the final asserted result.** A name containing "and" may still be OR in the SQL.

## Workflow

### 1. Collect test files

Find every file with test functions:

- Unit tests: `#[cfg(test)]` modules under `src/`.
- Integration tests: files under `tests/` directories.
- Use Grep for `#[test]` / `#[tokio::test]` / `#[rstest]` to enumerate candidates.

### 2. Read each file in FULL

- Do **not** stop at a Grep of function names. Each file must be opened with `Read`.
- If the volume is large, parallelize by delegating one file per `general-purpose` subagent. The delegation prompt MUST repeat the "Description Rules" below verbatim.

### 3. Description rules (per test)

For every test function:

1. Read its body.
2. Locate the assertion(s): `assert_eq!`, `assert!(... .is_ok())`, `assert!(... .is_err())`, `matches!(...)`, `.unwrap()` followed by a comparison, etc.
3. Translate the **expected state/value** of that assertion into a single Korean sentence.
4. Self-check:
   - Ignore the test name. Does the description still match what the assertion asserts?
   - If the description says "실패한다" / "성공한다", does it match `is_err()` / `is_ok()` in the body?
   - Is the concrete expected value (string literal, enum variant, number, empty vec, etc.) reflected in the description?
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
- Rewrite only descriptions that disagree with the current test body.
- Add rows for newly introduced tests.
- Remove rows for tests that no longer exist.

### 7. Verification (required before finishing)

1. Pick 3–5 tests at random, re-read their bodies, and diff against the written descriptions.
2. Run `markdownlint` expectations mentally — in particular, avoid MD025 (multiple H1).
3. Report to the user a summary of: rows added / rewritten / removed, and any tests whose intent is ambiguous enough to warrant a second look.

## Arguments (optional)

- No argument → process all test files.
- Path argument (e.g. `lib/src/domain`) → only refresh rows whose test files live under that path.
