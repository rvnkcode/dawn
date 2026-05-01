---
description: Review docs/class.md against the current codebase and report drift (fields, methods, relationships).
---

# Class Diagram Review

Verify that `docs/class.md` (Mermaid class diagram) faithfully reflects the current Rust codebase. Report drift; do **not** rewrite the diagram unless the user asks.

## Scope policy (do NOT flag the following as missing)

These omissions are **intentional**:

1. **Error types** — `AgeError`, `CliError`, `NextRowError`, `SQLiteError`, `UniqueIDParseError`, `IndexError`, `DescriptionEmptyError`, `TimestampError` and any other `*Error` enums/structs are deliberately excluded from the diagram. Do not propose adding them.
2. **Non-struct modules** — function-only helper modules (e.g. `outbound/query_builder.rs`) that do not expose a struct/enum/trait are deliberately excluded. Do not propose adding them.
3. **Std trait impls** — `Display`, `FromStr`, `Debug`, `Default`, `From`, `TryFrom`, `Clone`, `Eq`, `Hash`, `PartialEq`, etc. are deliberately not listed as members or `..|>` edges. Do not propose adding them.
4. **Derive-only traits** — `#[derive(Tabled)]` and similar derive-based trait implementations are not listed as `..|>` edges.
5. **Interface-covered dependencies** — When a class implements a trait via `..|>`, dependency edges (`..>` / `o--` / `*--`) for types already declared on the trait are intentionally omitted from the implementer. The trait owns the contract; the implementer only carries edges for behavior beyond the contract (e.g. types it constructs from raw data, or generates fresh) — and these may use a more specific verb than the trait's edge (e.g. `creates` / `queries` / `generates` instead of `accepts`). Do not propose adding edges that merely duplicate the trait's contract. This rule applies to trait implementations only; it does not weaken "No transitive rationalization" (section D) for field-borrowing across plain structs.

If the user expands scope in the prompt (e.g. "include errors this time"), override item 1 for that run only.

## What TO verify

For every class node in `docs/class.md`:

### A. Existence

- The struct/enum/trait exists in the codebase with the same name.
- It lives in the namespace (CLI / Domain / Outbound) the diagram places it in. Mismatch → report.

### B. Fields

- Field names and types match. `Option<T>`, `HashSet<T>`, `Vec<T>`, generic parameters must line up.
- Visibility markers (`+` public / `-` private / `~` crate) should roughly match Rust visibility (`pub` / private / `pub(crate)`). Don't nitpick `~` vs `-` when both are non-`pub`, but flag `+` on a non-`pub` member.
- A `#[cfg(test)]`-gated method or field must be either omitted or annotated — flag production-looking entries that are actually test-only.

### C. Methods

- Method name, receiver (`&self` / `&mut self` / consuming `self`), and return type (including `Result<_, E>`) match the source.
- Missing public methods on a listed class → report.
- Methods listed but not present in code → report.

### D. Relationships (edges)

Check each edge against the actual code:

| Edge syntax | Meaning | How to verify |
| --- | --- | --- |
| `A *-- B` | composition (A owns B directly) | struct A has field of type B (not `Option`, not behind a pointer) |
| `A o-- B` | aggregation (A holds optional / collection B) | struct A has `Option<B>`, `Vec<B>`, `HashSet<B>`, etc. |
| `A ..> B : <verb>` | dependency (A uses B) | A imports/takes/returns B; verb matches actual direction |
| `A ..\|> B : implements` | A implements trait B | `impl B for A` exists |

For labeled dependency edges, check that the **verb matches the actual interaction**:

- `accepts` — B appears as a parameter
- `returns` — B appears in the return type
- `creates` / `generates` — A constructs a B value
- `calls` — A invokes methods on B
- `implements` — reserved for `..|>`; do not use on `..>`
- `determines` / `targets` — stronger claims; only valid if A actively constructs or decides the value, not merely forwards it

Pass-through forwarding (A just hands B to a dependency) should be labeled `accepts`, not `determines` / `targets`.

**No transitive rationalization.** If a type appears in the class's own method/helper signatures or local typed bindings, it requires its own edge — even when it is also borrowed from another struct's field. Do not omit an edge with reasoning like "already covered transitively by the parent type's edge."

### E. Generics

- `Class~T~` generic parameter names should match the Rust definition.
- `where R is X` annotations on edges should match actual trait bounds.

## Workflow

1. Read `docs/class.md` end-to-end first.
2. For each namespace block in order (CLI → Domain → Outbound):
   - For each class node, open the corresponding source file with `Read` and verify sections A–C.
   - Enumerate the class file's `use` imports of domain/CLI types (skip error types per scope policy and stdlib/external crates). For each imported type, verify a corresponding outgoing edge (`..>` / `*--` / `o--` / `..|>`) from that class exists in the diagram. Missing edge → drift.
3. After all nodes, walk through every edge (lines matching `*--`, `o--`, `..>`, `..|>`) and verify section D.
4. Build a findings list. For each finding:
   - Quote the diagram line (path `docs/class.md:<line>`).
   - Cite the source line(s) that contradict it (`path:line`).
   - Propose the minimal fix (edit, not rewrite).

## Output

Report to the user in this shape:

```markdown
## Matches
- (brief bullets of non-obvious things that are correctly represented — keep short)

## Drift
### <Class or edge>
- **Diagram** (`docs/class.md:L<n>`): <quoted line>
- **Code** (`<path>:L<n>`): <actual signature / field>
- **Suggested fix**: <one-liner>
```

If there is no drift for a class, omit it entirely from the report. Close with a one-line summary: "N drift item(s) found" or "No drift found."

## Arguments (optional)

- No argument → review the whole diagram.
- A namespace name (`cli` / `domain` / `outbound`) → only review nodes and edges inside that namespace.
- A class name (e.g. `Filter`) → only review that class and edges touching it.
