use super::*;

mod crud;
mod init;
mod list;
mod triggers;

// Set in-memory database for testing
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
