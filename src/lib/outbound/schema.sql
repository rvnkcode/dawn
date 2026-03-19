CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY CHECK (length(id) = 11),
    description TEXT NOT NULL,
    entry INTEGER NOT NULL DEFAULT (unixepoch()),
    completed INTEGER,
    deleted INTEGER,
    modified INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Automatically update the 'modified' timestamp on task updates
CREATE TRIGGER IF NOT EXISTS trg_task_modified AFTER UPDATE ON task
WHEN
    old.description IS NOT new.description
    OR old.entry IS NOT new.entry
    OR old.completed IS NOT new.completed
    OR old.deleted IS NOT new.deleted
BEGIN
    UPDATE task
    SET modified = unixepoch()
    WHERE id = new.id;
END;

CREATE INDEX IF NOT EXISTS idx_task_pending
ON task (entry, id)
WHERE deleted IS NULL AND completed IS NULL;

CREATE VIEW IF NOT EXISTS vw_task_pending_row_id AS
SELECT
    id,
    row_number() OVER (ORDER BY entry, id) AS row_id
FROM task
WHERE deleted IS NULL AND completed IS NULL;

-- FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS task_fts USING fts5 (
    id,
    description,
    -- comment out next line when running sqlfluff lint of format
    tokenize = 'trigram remove_diacritics 1'
);

-- TODO: bigram tokenizer
-- https://www.space-i.com/post-blog/sqlite-fts-trigram-tokenizer%E3%81%A7unigram%EF%BC%86bigram%E6%A4%9C%E7%B4%A2%E3%81%BE%E3%81%A7%E3%82%B5%E3%83%9D%E3%83%BC%E3%83%88-%E6%97%A5%E6%9C%AC%E8%AA%9E%E5%85%A8%E6%96%87%E6%A4%9C%E7%B4%A2/
CREATE TRIGGER IF NOT EXISTS trg_task_fts_insert AFTER INSERT ON task
BEGIN
    INSERT INTO task_fts (id, description)
    VALUES (new.id, new.description);
END;

CREATE TRIGGER IF NOT EXISTS trg_task_fts_update AFTER UPDATE ON task
WHEN old.description IS NOT new.description
BEGIN
    UPDATE task_fts
    SET description = new.description
    WHERE id = new.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_task_fts_delete AFTER DELETE ON task
BEGIN
    DELETE FROM task_fts
    WHERE id = old.id;
END;

PRAGMA user_version = 1;
