use uuid::Uuid;

use super::{get_modified, insert_task, reset_modified, setup};
use crate::domain::task::{
    Description, Filter, Status, TaskCreation, TaskModification, Timestamp, port::TaskRepository,
};

// Create Task

#[test]
fn create_task_inserts_row() {
    let db = setup();
    let id = Uuid::new_v4();
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
    let id = Uuid::new_v4();
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
    insert_task(&db, "00000000-0000-0000-0000-000000000007", "task one");
    insert_task(&db, "00000000-0000-0000-0000-000000000008", "task two");
    insert_task(&db, "00000000-0000-0000-0000-000000000009", "task three");

    let count = db.count_pending().unwrap();

    assert_eq!(count, 3);
}

#[test]
fn count_pending_excludes_deleted_and_completed_tasks() {
    let db = setup();
    insert_task(&db, "00000000-0000-0000-0000-00000000000a", "pending one");
    insert_task(&db, "00000000-0000-0000-0000-00000000000b", "pending two");
    insert_task(&db, "00000000-0000-0000-0000-00000000000c", "pending three");
    insert_task(&db, "00000000-0000-0000-0000-00000000000d", "to complete");
    insert_task(&db, "00000000-0000-0000-0000-00000000000e", "to delete");

    db.conn
        .execute(
            "UPDATE task SET completed = unixepoch() WHERE id = ?1",
            rusqlite::params!["00000000-0000-0000-0000-00000000000d"],
        )
        .unwrap();
    db.conn
        .execute(
            "UPDATE task SET deleted = unixepoch() WHERE id = ?1",
            rusqlite::params!["00000000-0000-0000-0000-00000000000e"],
        )
        .unwrap();

    let count = db.count_pending().unwrap();

    assert_eq!(count, 3);
}

// Update Tasks

#[test]
fn update_tasks_changes_description() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-00000000003d".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-00000000003d", "original");

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
    let id: Uuid = "00000000-0000-0000-0000-00000000003e".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-00000000003e", "pending task");

    let modification = TaskModification {
        description: None,
        completed: Some(Some(Timestamp::new(1700000000).unwrap())),
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Completed))
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
    let id: Uuid = "00000000-0000-0000-0000-00000000003f".parse().unwrap();
    insert_task(
        &db,
        "00000000-0000-0000-0000-00000000003f",
        "completed task",
    );
    db.conn
        .execute(
            "UPDATE task SET completed = 1700000000 WHERE id = ?1",
            rusqlite::params!["00000000-0000-0000-0000-00000000003f"],
        )
        .unwrap();

    let modification = TaskModification {
        description: None,
        completed: Some(None),
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Pending))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].completed.is_none());
}

// IFNULL guard: a second `completed` write must not overwrite the original
// timestamp. Mirrors the row-level semantics of `completed = IFNULL(completed, ?)`
// in `build_update_clause`.
#[test]
fn update_tasks_preserves_existing_completed_timestamp() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-00000000004a".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-00000000004a", "task");
    db.update_tasks(
        &TaskModification {
            description: None,
            completed: Some(Some(Timestamp::new(1700000000).unwrap())),
            deleted: None,
        },
        &[id],
    )
    .unwrap();

    db.update_tasks(
        &TaskModification {
            description: None,
            completed: Some(Some(Timestamp::new(1900000000).unwrap())),
            deleted: None,
        },
        &[id],
    )
    .unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Completed))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks[0].completed,
        Some(Timestamp::new(1700000000).unwrap())
    );
}

// IFNULL guard for `deleted`: a second write must not overwrite the original
// timestamp. Mirrors `deleted = IFNULL(deleted, ?)`.
#[test]
fn update_tasks_preserves_existing_deleted_timestamp() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-00000000004b".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-00000000004b", "task");
    db.update_tasks(
        &TaskModification {
            description: None,
            completed: None,
            deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
        },
        &[id],
    )
    .unwrap();

    db.update_tasks(
        &TaskModification {
            description: None,
            completed: None,
            deleted: Some(Some(Timestamp::new(1900000000).unwrap())),
        },
        &[id],
    )
    .unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Deleted))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].deleted, Some(Timestamp::new(1700000000).unwrap()));
}

// In a batch update, IFNULL is evaluated per-row: rows without a prior
// timestamp receive the new one; rows with a prior value keep theirs.
#[test]
fn update_tasks_ifnull_is_per_row_in_batch_completed() {
    let db = setup();
    let id_existing: Uuid = "00000000-0000-0000-0000-00000000004c".parse().unwrap();
    let id_fresh: Uuid = "00000000-0000-0000-0000-00000000004d".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-00000000004c", "existing");
    insert_task(&db, "00000000-0000-0000-0000-00000000004d", "fresh");
    db.update_tasks(
        &TaskModification {
            description: None,
            completed: Some(Some(Timestamp::new(1700000000).unwrap())),
            deleted: None,
        },
        &[id_existing],
    )
    .unwrap();

    db.update_tasks(
        &TaskModification {
            description: None,
            completed: Some(Some(Timestamp::new(1900000000).unwrap())),
            deleted: None,
        },
        &[id_existing, id_fresh],
    )
    .unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Completed))
        .unwrap();
    let by_id: std::collections::HashMap<Uuid, Option<Timestamp>> =
        tasks.into_iter().map(|t| (t.uuid, t.completed)).collect();
    assert_eq!(
        by_id.get(&id_existing),
        Some(&Some(Timestamp::new(1700000000).unwrap()))
    );
    assert_eq!(
        by_id.get(&id_fresh),
        Some(&Some(Timestamp::new(1900000000).unwrap()))
    );
}

#[test]
fn update_tasks_sets_deleted() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-000000000041".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-000000000041", "pending task");

    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Deleted))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].deleted, Some(Timestamp::new(1700000000).unwrap()));
}

#[test]
fn update_tasks_clears_deleted() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-000000000042".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-000000000042", "deleted task");
    db.conn
        .execute(
            "UPDATE task SET deleted = 1700000000 WHERE id = ?1",
            rusqlite::params!["00000000-0000-0000-0000-000000000042"],
        )
        .unwrap();

    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(None),
    };
    db.update_tasks(&modification, &[id]).unwrap();

    let tasks = db
        .list_tasks(&Filter::default().with_report_status(Status::Pending))
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].deleted.is_none());
}

#[test]
fn update_tasks_fires_modified_trigger() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-000000000040".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-000000000040", "original");
    reset_modified(&db, "00000000-0000-0000-0000-000000000040");

    let modification = TaskModification {
        description: Some(Description::new("changed").unwrap()),
        completed: None,
        deleted: None,
    };
    db.update_tasks(&modification, &[id]).unwrap();

    assert!(get_modified(&db, "00000000-0000-0000-0000-000000000040") > 0);
}

// Delete Tasks

#[test]
fn delete_single_task() {
    let db = setup();
    let id: Uuid = "00000000-0000-0000-0000-000000000010".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-000000000010", "to be deleted");

    db.delete_tasks(&[id]).unwrap();

    let tasks = db.list_tasks(&Filter::default()).unwrap();
    assert!(tasks.is_empty());
}

#[test]
fn delete_multiple_tasks() {
    let db = setup();
    let id1: Uuid = "00000000-0000-0000-0000-000000000011".parse().unwrap();
    let id2: Uuid = "00000000-0000-0000-0000-000000000012".parse().unwrap();
    insert_task(&db, "00000000-0000-0000-0000-000000000010", "survivor");
    insert_task(&db, "00000000-0000-0000-0000-000000000011", "target one");
    insert_task(&db, "00000000-0000-0000-0000-000000000012", "target two");

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
