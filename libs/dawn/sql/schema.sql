DROP TABLE IF EXISTS task;

CREATE TABLE task (
    id TEXT PRIMARY KEY CHECK (length(id) = 11),
    description TEXT NOT NULL,
    completed_at NUMERIC,
    created_at NUMERIC DEFAULT (unixepoch()),
    updated_at NUMERIC DEFAULT (unixepoch()),
    deleted_at NUMERIC
);

-- Automatically update the updated_at timestamp
CREATE TRIGGER IF NOT EXISTS task_updated_at AFTER UPDATE ON task
WHEN
    old.description != new.description
    OR old.completed_at != new.completed_at
    OR old.deleted_at != new.deleted_at
BEGIN
    UPDATE task SET updated_at = unixepoch()
    WHERE id = new.id;
END;

PRAGMA user_version = 1;
