use rusqlite::params;

use super::{get_modified, insert_task, reset_modified, setup};

// Trigger: modified timestamp

#[test]
fn update_modified_when_description_changes() {
    let db = setup();
    let id = "00000000-0000-0000-0000-000000000001";
    insert_task(&db, id, "original");
    reset_modified(&db, id);

    db.conn
        .execute(
            "UPDATE task SET description = 'updated' WHERE id = ?1",
            params![id],
        )
        .unwrap();

    assert!(get_modified(&db, id) > 0);
}

#[test]
fn update_modified_when_completed_changes() {
    let db = setup();
    let id = "00000000-0000-0000-0000-000000000005";
    insert_task(&db, id, "task to complete");
    reset_modified(&db, id);

    db.conn
        .execute(
            "UPDATE task SET completed = unixepoch() WHERE id = ?1",
            params![id],
        )
        .unwrap();

    assert!(get_modified(&db, id) > 0);
}

#[test]
fn update_modified_when_deleted_changes() {
    let db = setup();
    let id = "00000000-0000-0000-0000-000000000006";
    insert_task(&db, id, "task to delete");
    reset_modified(&db, id);

    db.conn
        .execute(
            "UPDATE task SET deleted = unixepoch() WHERE id = ?1",
            params![id],
        )
        .unwrap();

    assert!(get_modified(&db, id) > 0);
}

#[test]
fn update_modified_when_entry_changes() {
    let db = setup();
    let id = "00000000-0000-0000-0000-000000000006";
    insert_task(&db, id, "task to re-date");
    reset_modified(&db, id);

    db.conn
        .execute(
            "UPDATE task SET entry = 1700000000 WHERE id = ?1",
            params![id],
        )
        .unwrap();

    assert!(get_modified(&db, id) > 0);
}

#[test]
fn not_update_modified_when_same_value() {
    let db = setup();
    let id = "00000000-0000-0000-0000-00000000000f";
    insert_task(&db, id, "unchanged");
    reset_modified(&db, id);

    db.conn
        .execute(
            "UPDATE task SET description = 'unchanged' WHERE id = ?1",
            params![id],
        )
        .unwrap();

    assert_eq!(get_modified(&db, id), 0);
}

// View: Pending Tasks Row ID

#[test]
fn assign_sequential_row_ids_to_pending_tasks() {
    let db = setup();
    insert_task(&db, "00000000-0000-0000-0000-000000000013", "first");
    insert_task(&db, "00000000-0000-0000-0000-000000000014", "second");
    insert_task(&db, "00000000-0000-0000-0000-000000000015", "third");

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
    assert_eq!(rows[0], ("00000000-0000-0000-0000-000000000013".into(), 1));
    assert_eq!(rows[1], ("00000000-0000-0000-0000-000000000014".into(), 2));
    assert_eq!(rows[2], ("00000000-0000-0000-0000-000000000015".into(), 3));
}

#[test]
fn exclude_deleted_and_completed_tasks_from_row_ids() {
    let db = setup();
    insert_task(&db, "00000000-0000-0000-0000-000000000016", "pending 1");
    insert_task(&db, "00000000-0000-0000-0000-000000000017", "to delete");
    insert_task(&db, "00000000-0000-0000-0000-000000000018", "pending 2");
    insert_task(&db, "00000000-0000-0000-0000-000000000019", "to complete");
    insert_task(&db, "00000000-0000-0000-0000-00000000001a", "pending 3");

    db.conn
        .execute(
            "UPDATE task SET deleted = unixepoch() WHERE id = ?1",
            params!["00000000-0000-0000-0000-000000000017"],
        )
        .unwrap();
    db.conn
        .execute(
            "UPDATE task SET completed = unixepoch() WHERE id = ?1",
            params!["00000000-0000-0000-0000-000000000019"],
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
    assert_eq!(rows[0], ("00000000-0000-0000-0000-000000000016".into(), 1));
    assert_eq!(rows[1], ("00000000-0000-0000-0000-000000000018".into(), 2));
    assert_eq!(rows[2], ("00000000-0000-0000-0000-00000000001a".into(), 3));
}

// FTS triggers

#[test]
fn sync_insert_to_fts() {
    let db = setup();
    insert_task(
        &db,
        "00000000-0000-0000-0000-00000000001b",
        "searchable task",
    );

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
    let id = "00000000-0000-0000-0000-00000000001c";
    insert_task(&db, id, "original text");

    db.conn
        .execute(
            "UPDATE task SET description = 'replacement text' WHERE id = ?1",
            params![id],
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
    let id = "00000000-0000-0000-0000-00000000001d";
    insert_task(&db, id, "ephemeral task");

    db.conn
        .execute("DELETE FROM task WHERE id = ?1", params![id])
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
