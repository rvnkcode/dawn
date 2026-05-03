use crate::outbound::sqlite::{SQLite, SQLiteError};

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
    assert_eq!(db.get_user_version().unwrap(), 1);
}

#[test]
fn error_when_database_version_is_newer() {
    let mut db = SQLite::new_in_memory().unwrap();
    db.conn.pragma_update(None, "user_version", 2).unwrap();

    let result = db.initialize();

    assert!(matches!(result, Err(SQLiteError::InitializationError(_))));
}
