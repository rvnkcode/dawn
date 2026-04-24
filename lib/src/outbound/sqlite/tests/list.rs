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

// Filter by Word

#[test]
fn list_tasks_filter_word_long_match() {
    let db = setup();
    let milk = Task {
        uid: "test_wlng001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let eggs = Task {
        uid: "test_wlng002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy eggs").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let mom = Task {
        uid: "test_wlng003".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("call mom").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &milk);
    insert_task_from(&db, &eggs);
    insert_task_from(&db, &mom);
    let filter = Filter::default().with_words(["milk".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![milk]);
}

#[test]
fn list_tasks_filter_word_long_case_insensitive() {
    let db = setup();
    let task = Task {
        uid: "test_wlng004".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("Buy MILK today").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_words(["milk".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![task]);
}

#[test]
fn list_tasks_filter_word_substring_match() {
    let db = setup();
    let task = Task {
        uid: "test_wlng005".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("refactoring code").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_words(["factor".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![task]);
}

#[test]
fn list_tasks_filter_word_no_match_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uid: "test_wlng006".parse().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("buy milk").unwrap(),
            entry: Timestamp::new(1000).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(1000).unwrap(),
        },
    );
    let filter = Filter::default().with_words(["xyz".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_word_short_match() {
    let db = setup();
    let target = Task {
        uid: "test_wsht001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("hi there").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uid: "test_wsht002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("go home").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &target);
    insert_task_from(&db, &other);
    let filter = Filter::default().with_words(["hi".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![target]);
}

#[test]
fn list_tasks_filter_word_short_case_insensitive() {
    let db = setup();
    let task = Task {
        uid: "test_wsht003".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("HI there").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_words(["hi".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![task]);
}

#[test]
fn list_tasks_filter_multiple_words_all_match() {
    let db = setup();
    let both = Task {
        uid: "test_wmlt001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk and eggs").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let bread = Task {
        uid: "test_wmlt002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy bread").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &both);
    insert_task_from(&db, &bread);
    let filter = Filter::default().with_words(["milk".to_string(), "eggs".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![both]);
}

#[test]
fn list_tasks_filter_multiple_words_partial_match_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uid: "test_wmlt003".parse().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("buy milk").unwrap(),
            entry: Timestamp::new(1000).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(1000).unwrap(),
        },
    );
    let filter = Filter::default().with_words(["milk".to_string(), "eggs".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_mixed_long_and_short_words() {
    let db = setup();
    let both = Task {
        uid: "test_wmlt004".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("hi there world").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let only_short = Task {
        uid: "test_wmlt005".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("hi nothing").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let only_long = Task {
        uid: "test_wmlt006".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("world only").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &both);
    insert_task_from(&db, &only_short);
    insert_task_from(&db, &only_long);
    let filter = Filter::default().with_words(["hi".to_string(), "world".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![both]);
}

#[test]
fn list_tasks_filter_word_korean_long() {
    let db = setup();
    let korean = Task {
        uid: "test_wkor001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("한글로 작성된 작업").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let english = Task {
        uid: "test_wkor002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("english task").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &korean);
    insert_task_from(&db, &english);
    let filter = Filter::default().with_words(["한글로".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![korean]);
}

#[test]
fn list_tasks_filter_word_korean_short() {
    let db = setup();
    let korean = Task {
        uid: "test_wkor003".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("한글 작업").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let english = Task {
        uid: "test_wkor004".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("english task").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &korean);
    insert_task_from(&db, &english);
    let filter = Filter::default().with_words(["한".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![korean]);
}

#[test]
fn list_tasks_filter_word_japanese_long() {
    let db = setup();
    let japanese = Task {
        uid: "test_wjpn001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("買い物に行く").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let english = Task {
        uid: "test_wjpn002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("english task").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &japanese);
    insert_task_from(&db, &english);
    let filter = Filter::default().with_words(["買い物".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![japanese]);
}

#[test]
fn list_tasks_filter_word_japanese_short() {
    let db = setup();
    let japanese = Task {
        uid: "test_wjpn003".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("買い物").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let english = Task {
        uid: "test_wjpn004".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("english task").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &japanese);
    insert_task_from(&db, &english);
    let filter = Filter::default().with_words(["買".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![japanese]);
}

#[test]
fn list_tasks_filter_word_japanese_hiragana_katakana() {
    let db = setup();
    let hiragana = Task {
        uid: "test_wjpn005".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("ひらがなのテスト").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let katakana_only = Task {
        uid: "test_wjpn006".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("カタカナのみ").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &hiragana);
    insert_task_from(&db, &katakana_only);
    let filter = Filter::default().with_words(["テスト".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![hiragana]);
}

#[test]
fn list_tasks_filter_word_with_like_metacharacter() {
    let db = setup();
    let percent = Task {
        uid: "test_wmta001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("50% off").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let plain = Task {
        uid: "test_wmta002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("abc off").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &percent);
    insert_task_from(&db, &plain);
    let filter = Filter::default().with_words(["%".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![percent]);
}

#[test]
fn list_tasks_filter_word_with_status() {
    let db = setup();
    let pending = Task {
        uid: "test_wsts001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uid: "test_wsts002".parse().unwrap(),
        index: None,
        description: Description::new("buy milk later").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &pending);
    insert_task_from(&db, &completed);
    let filter = Filter::default()
        .with_words(["milk".to_string()])
        .with_statuses([Status::Pending]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending]);
}

#[test]
fn list_tasks_filter_word_with_uid() {
    let db = setup();
    let target = Task {
        uid: "test_wuid001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uid: "test_wuid002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy milk too").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &target);
    insert_task_from(&db, &other);
    let filter = Filter::default()
        .with_uids(["test_wuid001".parse::<UniqueID>().unwrap()])
        .with_words(["milk".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![target]);
}

#[test]
fn list_tasks_filter_word_with_index() {
    let db = setup();
    let target = Task {
        uid: "test_widx001".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other_word = Task {
        uid: "test_widx002".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy bread").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let other_index = Task {
        uid: "test_widx003".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("buy milk too").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &target);
    insert_task_from(&db, &other_word);
    insert_task_from(&db, &other_index);
    let filter = Filter::default()
        .with_indices([Index::new(1).unwrap()])
        .with_words(["milk".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![target]);
}

#[test]
fn list_tasks_filter_word_with_index_short() {
    let db = setup();
    let target = Task {
        uid: "test_widx004".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("hi there").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uid: "test_widx005".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("hi friend").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &target);
    insert_task_from(&db, &other);
    let filter = Filter::default()
        .with_indices([Index::new(1).unwrap()])
        .with_words(["hi".to_string()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![target]);
}
