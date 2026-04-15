use super::{get_modified, insert_task, reset_modified, setup};
use crate::domain::task::{
    Description, Filter, Status, TaskCreation, TaskModification, Timestamp, UniqueID,
    port::TaskRepository,
};

// Create Task

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

// Count Pending

#[test]
fn count_pending_returns_error_when_table_missing() {
    let db = setup();
    db.conn.execute_batch("DROP TABLE task").unwrap();

    let result = db.count_pending();

    assert!(result.is_err());
}

#[test]
fn count_pending_returns_zero_when_no_tasks() {
    let db = setup();

    let count = db.count_pending().unwrap();

    assert_eq!(count, 0);
}

#[test]
fn count_pending_returns_count_of_pending_tasks() {
    let db = setup();
    insert_task(&db, "test_cnt0001", "task one");
    insert_task(&db, "test_cnt0002", "task two");
    insert_task(&db, "test_cnt0003", "task three");

    let count = db.count_pending().unwrap();

    assert_eq!(count, 3);
}

#[test]
fn count_pending_excludes_deleted_and_completed_tasks() {
    let db = setup();
    insert_task(&db, "test_cnt0004", "pending one");
    insert_task(&db, "test_cnt0005", "pending two");
    insert_task(&db, "test_cnt0006", "pending three");
    insert_task(&db, "test_cnt0007", "to complete");
    insert_task(&db, "test_cnt0008", "to delete");

    db.conn
        .execute(
            "UPDATE task SET completed = unixepoch() WHERE id = ?1",
            rusqlite::params!["test_cnt0007"],
        )
        .unwrap();
    db.conn
        .execute(
            "UPDATE task SET deleted = unixepoch() WHERE id = ?1",
            rusqlite::params!["test_cnt0008"],
        )
        .unwrap();

    let count = db.count_pending().unwrap();

    assert_eq!(count, 3);
}

// Update Tasks

#[test]
fn update_tasks_changes_description() {
    let db = setup();
    let id: UniqueID = "test_upd0001".parse().unwrap();
    insert_task(&db, "test_upd0001", "original");

    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: None,
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db.list_tasks(&Filter::default()).unwrap();
    assert_eq!(tasks[0].description, Description::new("updated").unwrap());
}

#[test]
fn update_tasks_sets_completed() {
    let db = setup();
    let id: UniqueID = "test_upd0002".parse().unwrap();
    insert_task(&db, "test_upd0002", "pending task");

    let modification = TaskModification {
        description: None,
        completed: Some(Some(Timestamp::new(1700000000).unwrap())),
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_statuses([Status::Completed]))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks[0].completed,
        Some(Timestamp::new(1700000000).unwrap())
    );
}

#[test]
fn update_tasks_clears_completed() {
    let db = setup();
    let id: UniqueID = "test_upd0003".parse().unwrap();
    insert_task(&db, "test_upd0003", "completed task");
    db.conn
        .execute(
            "UPDATE task SET completed = 1700000000 WHERE id = ?1",
            rusqlite::params!["test_upd0003"],
        )
        .unwrap();

    let modification = TaskModification {
        description: None,
        completed: Some(None),
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_statuses([Status::Pending]))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].completed.is_none());
}

#[test]
fn update_tasks_sets_deleted() {
    let db = setup();
    let id: UniqueID = "test_upd0006".parse().unwrap();
    insert_task(&db, "test_upd0006", "pending task");

    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_statuses([Status::Deleted]))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].deleted, Some(Timestamp::new(1700000000).unwrap()));
}

#[test]
fn update_tasks_clears_deleted() {
    let db = setup();
    let id: UniqueID = "test_upd0007".parse().unwrap();
    insert_task(&db, "test_upd0007", "deleted task");
    db.conn
        .execute(
            "UPDATE task SET deleted = 1700000000 WHERE id = ?1",
            rusqlite::params!["test_upd0007"],
        )
        .unwrap();

    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(None),
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_statuses([Status::Pending]))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].deleted.is_none());
}

#[test]
fn update_tasks_fires_modified_trigger() {
    let db = setup();
    let id: UniqueID = "test_upd0005".parse().unwrap();
    insert_task(&db, "test_upd0005", "original");
    reset_modified(&db, "test_upd0005");

    let modification = TaskModification {
        description: Some(Description::new("changed").unwrap()),
        completed: None,
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    assert!(get_modified(&db, "test_upd0005") > 0);
}

// Delete Tasks

#[test]
fn delete_single_task() {
    let db = setup();
    let id: UniqueID = "test_del0001".parse().unwrap();
    insert_task(&db, "test_del0001", "to be deleted");

    db.delete_tasks(&[id]).unwrap();

    let tasks = db.list_tasks(&Filter::default()).unwrap();
    assert!(tasks.is_empty());
}

#[test]
fn delete_multiple_tasks() {
    let db = setup();
    let id1: UniqueID = "test_del0002".parse().unwrap();
    let id2: UniqueID = "test_del0003".parse().unwrap();
    insert_task(&db, "test_del0001", "survivor");
    insert_task(&db, "test_del0002", "target one");
    insert_task(&db, "test_del0003", "target two");

    db.delete_tasks(&[id1, id2]).unwrap();

    let tasks = db.list_tasks(&Filter::default()).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, Description::new("survivor").unwrap());
}

#[test]
fn delete_empty_targets_returns_error() {
    let db = setup();

    let result = db.delete_tasks(&[]);

    assert!(result.is_err());
}
