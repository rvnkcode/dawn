DROP TABLE IF EXISTS task;

CREATE TABLE task (
    id TEXT PRIMARY KEY CHECK (length(id) = 11),
    description TEXT NOT NULL,
    completed_at NUMERIC,
    created_at NUMERIC DEFAULT (unixepoch()),
    updated_at NUMERIC DEFAULT (unixepoch()),
    deleted_at NUMERIC
);

CREATE TRIGGER IF NOT EXISTS task_updated_at AFTER UPDATE ON task
WHEN
    old.description != new.description
    OR old.completed_at != new.completed_at
    OR old.deleted_at != new.deleted_at
BEGIN
    UPDATE task SET updated_at = unixepoch()
    WHERE id = new.id;
END;

CREATE INDEX IF NOT EXISTS idx_task_pending
ON task (created_at, id)
WHERE deleted_at IS NULL AND completed_at IS NULL;

CREATE VIEW IF NOT EXISTS task_pending_row_id AS
SELECT
    id,
    row_number() OVER (ORDER BY created_at) AS row_id
FROM task
WHERE deleted_at IS NULL AND completed_at IS NULL;

-- FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS task_fts USING fts5 (
    id,
    description,
    -- comment out next line when running sqlfluff lint of format
    tokenize = 'trigram remove_diacritics 1'
);

-- TODO: bigram tokenizer
-- https://www.space-i.com/post-blog/sqlite-fts-trigram-tokenizer%E3%81%A7unigram%EF%BC%86bigram%E6%A4%9C%E7%B4%A2%E3%81%BE%E3%81%A7%E3%82%B5%E3%83%9D%E3%83%BC%E3%83%88-%E6%97%A5%E6%9C%AC%E8%AA%9E%E5%85%A8%E6%96%87%E6%A4%9C%E7%B4%A2/
CREATE TRIGGER IF NOT EXISTS task_fts_insert AFTER INSERT ON task
BEGIN
    INSERT INTO task_fts (id, description)
    VALUES (new.id, new.description);
END;

CREATE TRIGGER IF NOT EXISTS task_fts_update AFTER UPDATE ON task
BEGIN
    UPDATE task_fts
    SET description = new.description
    WHERE id = new.id;
END;

CREATE TRIGGER IF NOT EXISTS task_fts_delete AFTER DELETE ON task
BEGIN
    DELETE FROM task_fts
    WHERE id = old.id;
END;

PRAGMA user_version = 1;
