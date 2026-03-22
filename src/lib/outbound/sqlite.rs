use crate::domain::task::{TaskCreation, UniqueID, port::TaskRepository};
use rusqlite::Connection;
use std::cmp::Ordering;

const DB_VERSION: u8 = 1;

pub struct SQLite {
    conn: Connection,
}

impl SQLite {
    // Excluded from coverage because it depends on the filesystem
    #[cfg(not(coverage))]
    pub fn new() -> Result<Self, SQLiteError> {
        let db_path = get_db_path()?;
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub(crate) fn new_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    fn get_user_version(&self) -> Result<u8, SQLiteError> {
        let user_version = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(user_version)
    }

    pub fn initialize(&mut self) -> Result<(), SQLiteError> {
        let user_version = self.get_user_version()?;
        match user_version.cmp(&DB_VERSION) {
            Ordering::Less => {
                // TODO: Backup data
                let tx = self.conn.transaction()?;
                tx.execute_batch(include_str!("./schema.sql"))?;
                tx.commit()?;
                // TODO: Restore data
            }
            Ordering::Equal => {} // Do nothing
            Ordering::Greater => {
                return Err(SQLiteError::InitializationError(format!(
                    "database version ({user_version}) is newer than supported version ({DB_VERSION}). Please upgrade Dawn."
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SQLiteError {
    #[error("Database initialization error: {0}")]
    InitializationError(String),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

// Excluded from coverage because it depends on the filesystem
#[cfg(not(coverage))]
fn get_db_path() -> Result<std::path::PathBuf, SQLiteError> {
    /*
     * linux: ~/.local
     * macOS: ~/Library/Application\ Support
     */
    let data_dir = dirs::data_local_dir().ok_or(SQLiteError::InitializationError(
        "Could not determine local data directory".into(),
    ))?;
    let path = data_dir.join("dawn");
    std::fs::create_dir_all(&path).map_err(|e| {
        SQLiteError::InitializationError(format!("Could not create application directory: {}", e))
    })?;
    Ok(path.join("dawn.db"))
}

impl TaskRepository for SQLite {
    fn create_task(&self, id: &UniqueID, req: &TaskCreation) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO task (id, description) VALUES (?, ?)",
            [id.to_string(), req.description.to_string()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Description;

    // A. Initialize Database

    #[test]
    fn create_schema_on_fresh_database() {
        let mut db = SQLite::new_in_memory().unwrap();

        let result = db.initialize();

        assert!(result.is_ok());
        assert_eq!(db.get_user_version().unwrap(), 1);
    }

    #[test]
    fn skip_migration_when_version_matches() {
        let mut db = SQLite::new_in_memory().unwrap();
        db.initialize().unwrap();

        let result = db.initialize();

        assert!(result.is_ok());
    }

    #[test]
    fn error_when_database_version_is_newer() {
        let mut db = SQLite::new_in_memory().unwrap();
        db.conn.pragma_update(None, "user_version", 2).unwrap();

        let result = db.initialize();

        assert!(matches!(result, Err(SQLiteError::InitializationError(_))));
    }

    // Helper functions
    fn setup() -> SQLite {
        let mut db = SQLite::new_in_memory().unwrap();
        db.initialize().unwrap();
        db
    }

    fn insert_task(db: &SQLite, id: &str, description: &str) {
        db.conn
            .execute(
                "INSERT INTO task (id, description) VALUES (?1, ?2)",
                rusqlite::params![id, description],
            )
            .expect("insert_task: failed to insert test fixture");
    }

    fn get_modified(db: &SQLite, id: &str) -> i64 {
        db.conn
            .query_row(
                "SELECT modified FROM task WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .expect("get_modified: failed to query modified timestamp")
    }

    /* Sets `modified` to a sentinel value (0) without firing the trigger,
     * since the WHEN clause only watches description/entry/completed/deleted. */
    fn reset_modified(db: &SQLite, id: &str) {
        db.conn
            .execute(
                "UPDATE task SET modified = 0 WHERE id = ?1",
                rusqlite::params![id],
            )
            .expect("reset_modified: failed to set sentinel");
    }

    // B. Trigger: modified timestamp

    #[test]
    fn update_modified_when_description_changes() {
        let db = setup();
        let id = "test_aaaaaaa";
        insert_task(&db, id, "original");
        reset_modified(&db, id);

        db.conn
            .execute(
                "UPDATE task SET description = 'updated' WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

        assert!(get_modified(&db, id) > 0);
    }

    #[test]
    fn update_modified_when_completed_changes() {
        let db = setup();
        let id = "test_bbbbbbb";
        insert_task(&db, id, "task to complete");
        reset_modified(&db, id);

        db.conn
            .execute(
                "UPDATE task SET completed = unixepoch() WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

        assert!(get_modified(&db, id) > 0);
    }

    #[test]
    fn update_modified_when_deleted_changes() {
        let db = setup();
        let id = "test_ccccccc";
        insert_task(&db, id, "task to delete");
        reset_modified(&db, id);

        db.conn
            .execute(
                "UPDATE task SET deleted = unixepoch() WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

        assert!(get_modified(&db, id) > 0);
    }

    #[test]
    fn update_modified_when_entry_changes() {
        let db = setup();
        let id = "test_ccccccc";
        insert_task(&db, id, "task to redate");
        reset_modified(&db, id);

        db.conn
            .execute(
                "UPDATE task SET entry = 1700000000 WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

        assert!(get_modified(&db, id) > 0);
    }

    #[test]
    fn not_update_modified_when_same_value() {
        let db = setup();
        let id = "test_ddddddd";
        insert_task(&db, id, "unchanged");
        reset_modified(&db, id);

        db.conn
            .execute(
                "UPDATE task SET description = 'unchanged' WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

        assert_eq!(get_modified(&db, id), 0);
    }

    // C. View: Pending Tasks Row ID

    #[test]
    fn assign_sequential_row_ids_to_pending_tasks() {
        let db = setup();
        insert_task(&db, "test_eeeeee1", "first");
        insert_task(&db, "test_eeeeee2", "second");
        insert_task(&db, "test_eeeeee3", "third");

        let mut stmt = db
            .conn
            .prepare("SELECT id, row_id FROM vw_task_pending_row_id ORDER BY row_id")
            .unwrap();
        let rows: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], ("test_eeeeee1".into(), 1));
        assert_eq!(rows[1], ("test_eeeeee2".into(), 2));
        assert_eq!(rows[2], ("test_eeeeee3".into(), 3));
    }

    #[test]
    fn exclude_deleted_and_completed_tasks_from_row_ids() {
        let db = setup();
        insert_task(&db, "test_ffffff1", "pending 1");
        insert_task(&db, "test_ffffff2", "to delete");
        insert_task(&db, "test_ffffff3", "pending 2");
        insert_task(&db, "test_ffffff4", "to complete");
        insert_task(&db, "test_ffffff5", "pending 3");

        db.conn
            .execute(
                "UPDATE task SET deleted = unixepoch() WHERE id = ?1",
                rusqlite::params!["test_ffffff2"],
            )
            .unwrap();
        db.conn
            .execute(
                "UPDATE task SET completed = unixepoch() WHERE id = ?1",
                rusqlite::params!["test_ffffff4"],
            )
            .unwrap();

        let mut stmt = db
            .conn
            .prepare("SELECT id, row_id FROM vw_task_pending_row_id ORDER BY row_id")
            .unwrap();
        let rows: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], ("test_ffffff1".into(), 1));
        assert_eq!(rows[1], ("test_ffffff3".into(), 2));
        assert_eq!(rows[2], ("test_ffffff5".into(), 3));
    }

    // D. FTS triggers

    #[test]
    fn sync_insert_to_fts() {
        let db = setup();
        insert_task(&db, "test_ggggggg", "searchable task");

        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM task_fts WHERE task_fts MATCH 'searchable'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn sync_update_to_fts() {
        let db = setup();
        let id = "test_hhhhhhh";
        insert_task(&db, id, "original text");

        db.conn
            .execute(
                "UPDATE task SET description = 'replacement text' WHERE id = ?1",
                rusqlite::params![id],
            )
            .unwrap();

        let old_count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM task_fts WHERE task_fts MATCH 'original'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let new_count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM task_fts WHERE task_fts MATCH 'replacement'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(old_count, 0);
        assert_eq!(new_count, 1);
    }

    #[test]
    fn sync_delete_to_fts() {
        let db = setup();
        let id = "test_iiiiiii";
        insert_task(&db, id, "ephemeral task");

        db.conn
            .execute("DELETE FROM task WHERE id = ?1", rusqlite::params![id])
            .unwrap();

        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM task_fts WHERE task_fts MATCH 'ephemeral'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }

    // E. Create Task

    #[test]
    fn create_task_inserts_row() {
        let db = setup();
        let id = UniqueID::new();
        let req = TaskCreation {
            description: Description::new("buy milk").unwrap(),
        };

        let result = db.create_task(&id, &req);

        assert!(result.is_ok());
        let stored_desc: String = db
            .conn
            .query_row(
                "SELECT description FROM task WHERE id = ?1",
                rusqlite::params![id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_desc, "buy milk");
    }

    #[test]
    fn create_task_duplicate_id_returns_error() {
        let db = setup();
        let id = UniqueID::new();
        let req = TaskCreation {
            description: Description::new("first task").unwrap(),
        };
        db.create_task(&id, &req).unwrap();

        let req2 = TaskCreation {
            description: Description::new("second task").unwrap(),
        };
        let result = db.create_task(&id, &req2);

        assert!(result.is_err());
    }
}
