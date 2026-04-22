use super::setup;
use crate::domain::task::{
    Description, Filter, Index, Status, Task, Timestamp, UniqueID, port::TaskRepository,
};
use crate::outbound::sqlite::SQLite;

fn insert_task_from(db: &SQLite, task: &Task) {
    db.conn
        .execute(
            "INSERT INTO task (id, description, entry, completed, deleted, modified) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                task.uid.to_string(),
                task.description.to_string(),
                task.entry.as_seconds(),
                task.completed.as_ref().map(Timestamp::as_seconds),
                task.deleted.as_ref().map(Timestamp::as_seconds),
                task.modified.as_seconds(),
            ],
        )
        .expect("insert_task_from: failed to insert test fixture");
}

// List Tasks

#[test]
fn list_tasks_returns_empty_vec_when_no_tasks() {
    let db = setup();
    let filter = Filter::default();

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_orders_by_entry_then_id() {
    let db = setup();
    let earlier = Task {
        uid: "test_llllll3".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("earlier entry").unwrap(),
        entry: Timestamp::new(500).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(500).unwrap(),
    };
    let first_by_id = Task {
        uid: "test_llllll1".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("first by id").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let second_by_id = Task {
        uid: "test_llllll2".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("second by id").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &earlier);
    insert_task_from(&db, &first_by_id);
    insert_task_from(&db, &second_by_id);
    let filter = Filter::default();

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![earlier, first_by_id, second_by_id]);
}

#[test]
fn list_tasks_filter_pending_only() {
    let db = setup();
    let pending = Task {
        uid: "test_mmmmmm1".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_mmmmmm2".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uid: "test_mmmmmm3".parse().unwrap(),
        index: None,
        description: Description::new("deleted").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: Some(Timestamp::new(4000).unwrap()),
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &deleted);
    let filter = Filter::default().with_statuses([Status::Pending]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending]);
}

#[test]
fn list_tasks_filter_completed_only() {
    let db = setup();
    let pending = Task {
        uid: "test_nnnnnn1".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_nnnnnn2".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uid: "test_nnnnnn3".parse().unwrap(),
        index: None,
        description: Description::new("deleted").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: Some(Timestamp::new(4000).unwrap()),
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &deleted);
    let filter = Filter::default().with_statuses([Status::Completed]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![completed]);
}

#[test]
fn list_tasks_filter_deleted_only() {
    let db = setup();
    let pending = Task {
        uid: "test_oooooo1".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_oooooo2".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uid: "test_oooooo3".parse().unwrap(),
        index: None,
        description: Description::new("deleted").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: Some(Timestamp::new(4000).unwrap()),
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &deleted);
    let filter = Filter::default().with_statuses([Status::Deleted]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![deleted]);
}

#[test]
fn list_tasks_no_filter_returns_all() {
    let db = setup();
    let pending = Task {
        uid: "test_pppppp1".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_pppppp2".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uid: "test_pppppp3".parse().unwrap(),
        index: None,
        description: Description::new("deleted").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: Some(Timestamp::new(4000).unwrap()),
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &deleted);
    let filter = Filter::default();

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending, completed, deleted]);
}

#[test]
fn list_tasks_filter_two_statuses() {
    let db = setup();
    let pending = Task {
        uid: "test_qqqqqq1".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_qqqqqq2".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uid: "test_qqqqqq3".parse().unwrap(),
        index: None,
        description: Description::new("deleted").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: Some(Timestamp::new(4000).unwrap()),
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &deleted);
    let filter = Filter::default().with_statuses([Status::Pending, Status::Completed]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending, completed]);
}

#[test]
fn list_tasks_filter_all_statuses() {
    let db = setup();
    let pending = Task {
        uid: "test_rrrrrr1".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_rrrrrr2".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uid: "test_rrrrrr3".parse().unwrap(),
        index: None,
        description: Description::new("deleted").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: Some(Timestamp::new(4000).unwrap()),
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &deleted);
    let filter =
        Filter::default().with_statuses([Status::Pending, Status::Completed, Status::Deleted]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending, completed, deleted]);
}

// Filter by UID

#[test]
fn list_tasks_filter_single_uid() {
    let db = setup();
    let target = Task {
        uid: "test_sssss01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("target").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other1 = Task {
        uid: "test_sssss02".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("other 1").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let other2 = Task {
        uid: "test_sssss03".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("other 2").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &target);
    insert_task_from(&db, &other1);
    insert_task_from(&db, &other2);
    let filter = Filter::default().with_uids(["test_sssss01".parse::<UniqueID>().unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![target]);
}

#[test]
fn list_tasks_filter_multiple_uids() {
    let db = setup();
    let first = Task {
        uid: "test_ttttt01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("first").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let second = Task {
        uid: "test_ttttt02".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("second").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uid: "test_ttttt03".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("excluded").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &first);
    insert_task_from(&db, &second);
    insert_task_from(&db, &excluded);
    let filter = Filter::default().with_uids([
        "test_ttttt01".parse::<UniqueID>().unwrap(),
        "test_ttttt02".parse::<UniqueID>().unwrap(),
    ]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![first, second]);
}

#[test]
fn list_tasks_filter_nonexistent_uid() {
    let db = setup();
    let task = Task {
        uid: "test_vvvvv01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("existing").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let nonexistent: UniqueID = "test_vvvvv99".parse().unwrap();
    let filter = Filter::default().with_uids([nonexistent]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_uid_with_status() {
    let db = setup();
    let pending = Task {
        uid: "test_uuuuu01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_uuuuu02".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let other = Task {
        uid: "test_uuuuu03".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("other pending").unwrap(),
        entry: Timestamp::new(4000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(4000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &other);
    let filter = Filter::default()
        .with_uids([
            "test_uuuuu01".parse::<UniqueID>().unwrap(),
            "test_uuuuu02".parse::<UniqueID>().unwrap(),
        ])
        .with_statuses([Status::Pending]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending]);
}

// Filter by Index

#[test]
fn list_tasks_filter_single_index() {
    let db = setup();
    let target = Task {
        uid: "test_wwwww01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("target").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uid: "test_wwwww02".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("other").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &target);
    insert_task_from(&db, &other);
    let filter = Filter::default().with_indices([Index::new(1).unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![target]);
}

#[test]
fn list_tasks_filter_multiple_indices() {
    let db = setup();
    let first = Task {
        uid: "test_xxxxx01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("first").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let second = Task {
        uid: "test_xxxxx02".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("second").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uid: "test_xxxxx03".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("excluded").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &first);
    insert_task_from(&db, &second);
    insert_task_from(&db, &excluded);
    let filter = Filter::default().with_indices([Index::new(1).unwrap(), Index::new(2).unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![first, second]);
}

#[test]
fn list_tasks_filter_index_with_completed_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uid: "test_yyyzz01".parse().unwrap(),
            index: None,
            description: Description::new("completed").unwrap(),
            entry: Timestamp::new(1000).unwrap(),
            completed: Some(Timestamp::new(2000).unwrap()),
            deleted: None,
            modified: Timestamp::new(1000).unwrap(),
        },
    );
    insert_task_from(
        &db,
        &Task {
            uid: "test_yyyzz02".parse().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("pending").unwrap(),
            entry: Timestamp::new(3000).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(3000).unwrap(),
        },
    );
    let filter = Filter::default()
        .with_indices([Index::new(1).unwrap()])
        .with_statuses([Status::Completed]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_index_with_deleted_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uid: "test_yyyzz03".parse().unwrap(),
            index: None,
            description: Description::new("deleted").unwrap(),
            entry: Timestamp::new(1000).unwrap(),
            completed: None,
            deleted: Some(Timestamp::new(2000).unwrap()),
            modified: Timestamp::new(1000).unwrap(),
        },
    );
    insert_task_from(
        &db,
        &Task {
            uid: "test_yyyzz04".parse().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("pending").unwrap(),
            entry: Timestamp::new(3000).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(3000).unwrap(),
        },
    );
    let filter = Filter::default()
        .with_indices([Index::new(1).unwrap()])
        .with_statuses([Status::Deleted]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_nonexistent_index() {
    let db = setup();
    let task = Task {
        uid: "test_zzzzz01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("existing").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_indices([Index::new(99).unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

// Filter by UID + Index

#[test]
fn list_tasks_filter_uid_and_index() {
    let db = setup();
    let by_uid = Task {
        uid: "test_aaaab01".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("matched by uid").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let by_index = Task {
        uid: "test_aaaab02".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("matched by index").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uid: "test_aaaab03".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("excluded").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &by_uid);
    insert_task_from(&db, &by_index);
    insert_task_from(&db, &excluded);
    let filter = Filter::default()
        .with_uids(["test_aaaab01".parse::<UniqueID>().unwrap()])
        .with_indices([Index::new(2).unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![by_uid, by_index]);
}
