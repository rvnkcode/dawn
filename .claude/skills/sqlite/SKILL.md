---
name: sqlite
description: SQLite database patterns for query optimization, schema design, indexing, and Rust integration. Best practices for embedded databases.
---

# SQLite Patterns

Quick reference for SQLite best practices in Rust applications.

## When to Activate

- Writing SQL queries or migrations for SQLite
- Designing database schemas
- Troubleshooting slow queries
- Configuring WAL mode and concurrency
- Using rusqlite or sqlx with SQLite
- Implementing full-text search (FTS5)
- Supporting CJK (Chinese, Japanese, Korean) text search

## Quick Reference

### Index Cheat Sheet

| Query Pattern | Index Type | Example |
| ------------- | ----------- | ------- |
| `WHERE col = value` | B-tree (default) | `CREATE INDEX idx ON t (col)` |
| `WHERE col > value` | B-tree | `CREATE INDEX idx ON t (col)` |
| `WHERE a = x AND b > y` | Composite | `CREATE INDEX idx ON t (a, b)` |
| `ORDER BY col` | B-tree | `CREATE INDEX idx ON t (col)` |
| Uniqueness | Unique | `CREATE UNIQUE INDEX idx ON t (col)` |

**Note:** SQLite only supports B-tree indexes (no GIN, BRIN, Hash).

### Data Type Quick Reference

SQLite uses dynamic typing with 5 storage classes:

| Storage Class | Use Case | Example |
|---------------|----------|---------|
| `TEXT` | UUIDs, strings, dates | `id TEXT PRIMARY KEY` |
| `INTEGER` | Counts, booleans, enums | `priority INTEGER` |
| `REAL` | Floating point | `latitude REAL` |
| `BLOB` | Binary data, UUID (compact) | `file_data BLOB` |
| `NULL` | Missing values | - |

**Type Affinity Rules:**

```sql
-- Recommended explicit types
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,              -- UUID (UUIDv7 recommended)
  description TEXT NOT NULL,
  priority INTEGER DEFAULT 0,       -- 0-9 scale
  due_date TEXT,                    -- ISO8601: '2024-01-15T09:00:00Z'
  is_completed INTEGER DEFAULT 0,   -- Boolean: 0 or 1
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  metadata TEXT                     -- JSON as text
);
```

### Common Patterns

**Primary Key Strategy (UUID):**

```sql
-- ✅ RECOMMENDED: UUIDv7 as TEXT PRIMARY KEY
-- UUIDv7 is time-ordered, so it maintains insertion order and index efficiency
CREATE TABLE tasks (
  id TEXT PRIMARY KEY CHECK (length(id) = 36),  -- UUID format validation
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  ...
);

-- ✅ GOOD: BLOB for storage efficiency (16 bytes vs 36 bytes TEXT)
CREATE TABLE tasks (
  id BLOB PRIMARY KEY CHECK (length(id) = 16),
  ...
);

-- ❌ BAD: UUIDv4 (random) causes index fragmentation on high-volume inserts
-- Use UUIDv7 (time-ordered) instead
```

**UUID Format Comparison:**

| Format | Size | Pros | Cons |
|--------|------|------|------|
| TEXT (hyphenated) | 36 bytes | Human-readable, debuggable | Larger storage |
| TEXT (no hyphens) | 32 bytes | Smaller than hyphenated | Less readable |
| BLOB | 16 bytes | Most compact, fastest | Not human-readable |

**Why UUIDv7 over UUIDv4:**

- Time-ordered: maintains B-tree index efficiency
- No index fragmentation from random inserts
- Sortable by creation time
- Still globally unique

**Composite Index Order:**

```sql
-- Equality columns first, then range columns
CREATE INDEX idx_tasks_status_due ON tasks (status, due_date);
-- Works for: WHERE status = 'pending' AND due_date > '2024-01-01'
```

**Covering Index:**

```sql
-- Include all SELECT columns to avoid table lookup
CREATE INDEX idx_tasks_project ON tasks (project_id, status, due_date);
-- SELECT status, due_date FROM tasks WHERE project_id = 1
```

**Partial Index:**

```sql
CREATE INDEX idx_tasks_pending ON tasks (due_date) WHERE status = 'pending';
-- Smaller index, only includes pending tasks
```

**UPSERT:**

```sql
INSERT INTO tasks (uuid, description, status)
VALUES ('abc-123', 'New task', 'pending')
ON CONFLICT (uuid)
DO UPDATE SET
  description = excluded.description,
  modified_at = datetime('now');
```

**Cursor Pagination:**

```sql
-- ✅ GOOD: O(1) performance with UUIDv7 (time-ordered)
SELECT * FROM tasks WHERE id > ?last_id ORDER BY id LIMIT 20;

-- ✅ GOOD: Using created_at for pagination
SELECT * FROM tasks
WHERE created_at < ?last_created_at
ORDER BY created_at DESC
LIMIT 20;

-- ❌ BAD: O(n) performance
SELECT * FROM tasks ORDER BY id LIMIT 20 OFFSET 1000;
```

### WAL Mode (Critical for Concurrency)

```sql

-- Enable WAL mode (do once, persists)
PRAGMA journal_mode = WAL;

-- Configure WAL checkpointing
PRAGMA wal_autocheckpoint = 1000;  -- Pages between checkpoints

-- Synchronous mode (trade durability for speed)
PRAGMA synchronous = NORMAL;  -- FULL (safest) | NORMAL | OFF (fastest)
```

**WAL Benefits:**

- Concurrent reads during writes
- Better write performance
- Crash recovery

### Performance PRAGMAs

```sql
-- Memory settings
PRAGMA cache_size = -64000;        -- 64MB page cache
PRAGMA temp_store = MEMORY;        -- Temp tables in memory
PRAGMA mmap_size = 268435456;      -- 256MB memory-mapped I/O

-- Analysis
PRAGMA optimize;                   -- Run periodically
ANALYZE;                           -- Update query planner statistics

-- Integrity
PRAGMA integrity_check;            -- Full database check
PRAGMA quick_check;                -- Fast integrity check
```

### Query Analysis

```sql
-- Explain query plan
EXPLAIN QUERY PLAN SELECT * FROM tasks WHERE project_id = 1;

-- Look for:
-- ✅ SEARCH ... USING INDEX
-- ❌ SCAN (full table scan)
```

### Foreign Keys (Disabled by Default!)

```sql
-- Enable foreign keys (must do per connection!)
PRAGMA foreign_keys = ON;

-- Define with proper actions (UUID foreign keys)
CREATE TABLE subtasks (
  id TEXT PRIMARY KEY CHECK (length(id) = 36),
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  description TEXT NOT NULL
);

-- Always index foreign key columns!
CREATE INDEX idx_subtasks_task_id ON subtasks (task_id);
```

### Date/Time Handling

```sql
-- Store as ISO8601 TEXT
INSERT INTO tasks (due_date) VALUES ('2024-01-15T09:00:00Z');

-- Date functions
SELECT * FROM tasks WHERE date(due_date) = date('now');
SELECT * FROM tasks WHERE due_date > datetime('now', '-7 days');
SELECT * FROM tasks WHERE due_date BETWEEN
  datetime('now', 'start of day') AND datetime('now', '+1 day', 'start of day');
```

### JSON Support (SQLite 3.38+)

```sql
-- Store JSON
UPDATE tasks SET metadata = '{"tags": ["work", "urgent"]}';

-- Query JSON
SELECT * FROM tasks WHERE json_extract(metadata, '$.tags') LIKE '%urgent%';
SELECT * FROM tasks WHERE metadata->>'$.priority' = 'high';

-- Index JSON paths
CREATE INDEX idx_tasks_priority ON tasks (json_extract(metadata, '$.priority'));
```

---

## Full-Text Search (FTS5)

FTS5 is SQLite's full-text search engine. Much faster than `LIKE '%term%'` for text search.

### Creating FTS5 Tables

```sql
-- Basic FTS5 table
CREATE VIRTUAL TABLE tasks_fts USING fts5(
  title,
  description,
  content='tasks',        -- External content table
  content_rowid='rowid'   -- Link to source table
);

-- With tokenizer options
CREATE VIRTUAL TABLE tasks_fts USING fts5(
  title,
  description,
  tokenize='porter unicode61',  -- Stemming + Unicode support
  prefix='2 3'                  -- Index 2 and 3 character prefixes
);
```

### Built-in Tokenizers

| Tokenizer | Use Case | Example |
|-----------|----------|---------|
| `unicode61` | Default, multilingual | `tokenize='unicode61'` |
| `porter` | English stemming (run→running) | `tokenize='porter unicode61'` |
| `ascii` | ASCII-only, faster | `tokenize='ascii'` |
| `trigram` | Substring matching | `tokenize='trigram'` |

### Query Syntax

```sql
-- Basic search (implicit AND)
SELECT * FROM tasks_fts WHERE tasks_fts MATCH 'project report';

-- Phrase search
SELECT * FROM tasks_fts WHERE tasks_fts MATCH '"quarterly report"';

-- Boolean operators
SELECT * FROM tasks_fts WHERE tasks_fts MATCH 'project OR meeting';
SELECT * FROM tasks_fts WHERE tasks_fts MATCH 'project NOT archived';

-- Prefix search
SELECT * FROM tasks_fts WHERE tasks_fts MATCH 'proj*';

-- Column-specific search
SELECT * FROM tasks_fts WHERE tasks_fts MATCH 'title:urgent';

-- NEAR query (words within 5 tokens)
SELECT * FROM tasks_fts WHERE tasks_fts MATCH 'NEAR(deadline project, 5)';

-- Initial token match (at column start)
SELECT * FROM tasks_fts WHERE tasks_fts MATCH '^URGENT';
```

### BM25 Ranking

```sql
-- Order by relevance (lower = more relevant)
SELECT *, bm25(tasks_fts) AS rank
FROM tasks_fts
WHERE tasks_fts MATCH 'project'
ORDER BY rank;

-- Column weights (title more important than description)
SELECT *, bm25(tasks_fts, 10.0, 1.0) AS rank
FROM tasks_fts
WHERE tasks_fts MATCH 'urgent'
ORDER BY rank;
```

### External Content Tables (Recommended)

Keep FTS index in sync with main table using triggers:

```sql
-- Main table
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  rowid INTEGER  -- Needed for FTS content linking
);

-- FTS table pointing to main table
CREATE VIRTUAL TABLE tasks_fts USING fts5(
  title,
  description,
  content='tasks',
  content_rowid='rowid'
);

-- Sync triggers
CREATE TRIGGER tasks_ai AFTER INSERT ON tasks BEGIN
  INSERT INTO tasks_fts(rowid, title, description)
  VALUES (new.rowid, new.title, new.description);
END;

CREATE TRIGGER tasks_ad AFTER DELETE ON tasks BEGIN
  INSERT INTO tasks_fts(tasks_fts, rowid, title, description)
  VALUES ('delete', old.rowid, old.title, old.description);
END;

CREATE TRIGGER tasks_au AFTER UPDATE ON tasks BEGIN
  INSERT INTO tasks_fts(tasks_fts, rowid, title, description)
  VALUES ('delete', old.rowid, old.title, old.description);
  INSERT INTO tasks_fts(rowid, title, description)
  VALUES (new.rowid, new.title, new.description);
END;

-- Rebuild index if out of sync
INSERT INTO tasks_fts(tasks_fts) VALUES ('rebuild');
```

### FTS5 Performance Tips

```sql
-- Optimize for fastest queries (merge all b-trees)
INSERT INTO tasks_fts(tasks_fts) VALUES ('optimize');

-- Reduce storage with columnsize=0 (if you don't need xColumnSize)
CREATE VIRTUAL TABLE t USING fts5(content, columnsize=0);

-- Use detail=column to save space (omit position info)
CREATE VIRTUAL TABLE t USING fts5(content, detail=column);

-- Automerge configuration (balance write speed vs query speed)
INSERT INTO tasks_fts(tasks_fts, rank) VALUES ('automerge', 4);
```

---

## CJK Full-Text Search (中文/日本語/한국어)

The built-in tokenizers don't work well with CJK languages because they rely on whitespace to separate words. CJK languages don't use spaces between words.

### The Problem

```sql
-- ❌ unicode61 tokenizer treats CJK text as single tokens
-- "안녕하세요" becomes one token, not searchable by "안녕"
CREATE VIRTUAL TABLE t USING fts5(content, tokenize='unicode61');
INSERT INTO t VALUES ('안녕하세요 반갑습니다');
SELECT * FROM t WHERE t MATCH '안녕';  -- No results!
```

### Solution 1: Trigram Tokenizer (Simple, Built-in)

Creates 3-character sliding window tokens. Works for substring matching.

```sql
CREATE VIRTUAL TABLE tasks_fts USING fts5(
  title,
  description,
  tokenize='trigram'
);

-- Now substring search works
INSERT INTO tasks_fts VALUES ('프로젝트 보고서', '분기별 실적 보고');
SELECT * FROM tasks_fts WHERE tasks_fts MATCH '프로젝트';  -- ✅ Works!
SELECT * FROM tasks_fts WHERE tasks_fts MATCH '보고';      -- ✅ Works!
```

**Limitations:**

- Minimum 3 characters for matching
- Larger index size than word-based tokenizers
- No stemming or linguistic analysis

### Solution 2: better-trigram (Enhanced CJK Support)

Treats each CJK character as its own token while using trigrams for non-CJK text.

```sql
-- Load extension first
.load ./libsqlite_better_trigram

CREATE VIRTUAL TABLE tasks_fts USING fts5(
  content,
  tokenize='better_trigram'
);

-- "李红：那是钢笔" tokenizes as ['李','红','：','那','是','钢','笔']
-- Single character search works!
SELECT * FROM tasks_fts WHERE tasks_fts MATCH '钢';  -- ✅ Works!
```

### Solution 3: lindera-sqlite (Best for Japanese/Korean/Chinese)

Proper morphological analysis using language-specific dictionaries.

```sql
-- Load Lindera extension
.load ./liblindera_sqlite lindera_fts5_tokenizer_init

CREATE VIRTUAL TABLE tasks_fts USING fts5(
  content,
  tokenize='lindera_tokenizer'
);

-- Proper word segmentation
-- "東京都に住んでいます" → ["東京都", "に", "住んで", "います"]
INSERT INTO tasks_fts VALUES ('東京都に住んでいます');
SELECT * FROM tasks_fts WHERE tasks_fts MATCH '東京都';  -- ✅ Correct match!
```

**Cargo.toml for lindera-sqlite:**

```toml
[dependencies]
lindera-sqlite = { version = "2", features = ["embed-cjk"] }
# Or specific dictionaries:
# features = ["embed-ipadic"]     # Japanese
# features = ["embed-ko-dic"]     # Korean
# features = ["embed-cc-cedict"]  # Chinese
```

### CJK Tokenizer Comparison

| Tokenizer | Languages | Accuracy | Index Size | Setup |
|-----------|-----------|----------|------------|-------|
| `trigram` | All | Low | Large | Built-in |
| `better_trigram` | CJK | Medium | Medium | Extension |
| `lindera` | JA/KO/ZH | High | Small | Extension + Dict |

### Rust FTS5 Integration

```rust
use rusqlite::{Connection, Result};

fn search_tasks(conn: &Connection, query: &str) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.id, t.title, t.description, bm25(tasks_fts) as rank
         FROM tasks_fts
         JOIN tasks t ON tasks_fts.rowid = t.rowid
         WHERE tasks_fts MATCH ?1
         ORDER BY rank
         LIMIT 20"
    )?;

    let tasks = stmt.query_map([query], |row| {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(tasks)
}

// Escape special FTS5 characters in user input
fn escape_fts_query(query: &str) -> String {
    // Wrap in quotes for phrase search, escape internal quotes
    format!("\"{}\"", query.replace('"', "\"\""))
}
```

---

## Rust Integration

### rusqlite Patterns

```rust
use rusqlite::{Connection, params, Result};
use uuid::Uuid;

// Open with recommended settings
fn open_database(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Essential PRAGMAs
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA cache_size = -64000;
        PRAGMA temp_store = MEMORY;
    ")?;

    Ok(conn)
}

// UUID generation (use uuid crate with v7 feature)
fn new_id() -> String {
    Uuid::now_v7().to_string()  // Time-ordered UUID
}

// Parameterized queries with UUID
fn find_tasks_by_status(conn: &Connection, status: &str) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, description, status FROM tasks WHERE status = ?1"
    )?;

    let tasks = stmt.query_map([status], |row| {
        Ok(Task {
            id: row.get(0)?,  // Returns String (UUID)
            description: row.get(1)?,
            status: row.get(2)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(tasks)
}

// Transaction handling with UUID
fn create_task_with_subtasks(
    conn: &mut Connection,
    task: &Task,
    subtasks: &[Subtask],
) -> Result<String> {
    let tx = conn.transaction()?;
    let task_id = new_id();

    tx.execute(
        "INSERT INTO tasks (id, description, status) VALUES (?1, ?2, ?3)",
        params![task_id, task.description, task.status],
    )?;

    for subtask in subtasks {
        let subtask_id = new_id();
        tx.execute(
            "INSERT INTO subtasks (id, task_id, description) VALUES (?1, ?2, ?3)",
            params![subtask_id, task_id, subtask.description],
        )?;
    }

    tx.commit()?;
    Ok(task_id)
}
```

**Cargo.toml:**

```toml
[dependencies]
uuid = { version = "1", features = ["v7"] }
rusqlite = { version = "0.31", features = ["bundled"] }
```

### sqlx Async Patterns

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| Box::pin(async move {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(conn).await?;
            Ok(())
        }))
        .connect(database_url).await
}

// Compile-time verified queries with UUID
async fn find_task(pool: &SqlitePool, id: &str) -> Result<Task, sqlx::Error> {
    sqlx::query_as!(
        Task,
        "SELECT id, description, status FROM tasks WHERE id = ?",
        id
    )
    .fetch_one(pool)
    .await
}

async fn create_task(pool: &SqlitePool, description: &str) -> Result<String, sqlx::Error> {
    let id = Uuid::now_v7().to_string();

    sqlx::query!(
        "INSERT INTO tasks (id, description, status) VALUES (?, ?, 'pending')",
        id,
        description
    )
    .execute(pool)
    .await?;

    Ok(id)
}
```

## Anti-Pattern Detection

```sql
-- Find tables without primary key (bad practice)
SELECT name FROM sqlite_master
WHERE type = 'table'
  AND name NOT LIKE 'sqlite_%'
  AND sql NOT LIKE '%PRIMARY KEY%';

-- Find unindexed foreign key columns
-- (Manual check - SQLite doesn't track FK relationships in system tables)

-- Check index usage after queries
EXPLAIN QUERY PLAN SELECT * FROM tasks WHERE project_id = 1;
-- Look for SCAN instead of SEARCH
```

## Anti-Patterns to Avoid

### ❌ Query Anti-Patterns

- `SELECT *` in production code
- Missing indexes on WHERE/JOIN columns
- OFFSET pagination on large tables
- Not using `prepare_cached()` for repeated queries
- String concatenation for queries (SQL injection)
- Using `LIKE '%term%'` instead of FTS5 for text search
- Not escaping user input in FTS5 MATCH queries

### ❌ Schema Anti-Patterns

- UUIDv4 (random) as primary key on high-volume tables (causes index fragmentation)
- Not enabling foreign keys
- Storing dates as non-ISO8601 formats
- Using REAL for money (use INTEGER cents)
- Missing indexes on UUID foreign key columns
- Using `unicode61` tokenizer for CJK text (use trigram or lindera)
- FTS5 table without external content (data duplication)

### ❌ Configuration Anti-Patterns

- Not enabling WAL mode
- Default cache_size (too small)
- PRAGMA foreign_keys not set per connection
- Not running ANALYZE periodically

### ❌ Concurrency Anti-Patterns

- Long-running transactions blocking writers
- Not handling SQLITE_BUSY errors
- Opening multiple write connections without pooling

## Checklist

### Before Deploying SQLite Changes:

- [ ] WAL mode enabled
- [ ] Foreign keys enabled
- [ ] All WHERE/JOIN columns indexed
- [ ] Foreign key columns indexed
- [ ] Composite indexes in correct order
- [ ] `EXPLAIN QUERY PLAN` shows SEARCH not SCAN
- [ ] Parameterized queries (no string concat)
- [ ] Transactions for multi-statement operations
- [ ] `ANALYZE` run after bulk data changes
- [ ] UUIDv7 used for primary keys (time-ordered)

### FTS5 Checklist:

- [ ] External content table with sync triggers
- [ ] Appropriate tokenizer for language (trigram/lindera for CJK)
- [ ] User input escaped before MATCH queries
- [ ] BM25 ranking with appropriate column weights
- [ ] `optimize` run after bulk inserts

---

*Patterns for SQLite 3.35+ with modern features (RETURNING, UPSERT, JSON, FTS5)*

## References

- [SQLite FTS5 Extension](https://sqlite.org/fts5.html)
- [lindera-sqlite](https://lib.rs/crates/lindera-sqlite) - CJK tokenizer for Rust
- [sqlite-better-trigram](https://github.com/streetwriters/sqlite-better-trigram) - Enhanced CJK support
