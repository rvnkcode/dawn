use uuid::Uuid;

use super::setup;
use crate::{
    domain::task::{
        Description, Filter, Index, IndexRange, Status, Task, Timestamp, UuidPrefix,
        port::TaskRepository,
    },
    outbound::sqlite::SQLite,
};

fn insert_task_from(db: &SQLite, task: &Task) {
    db.conn
        .execute(
            "INSERT INTO task (id, description, entry, completed, deleted, modified) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                task.uuid.to_string(),
                task.description.to_string(),
                task.entry.as_seconds(),
                task.completed.as_ref().map(Timestamp::as_seconds),
                task.deleted.as_ref().map(Timestamp::as_seconds),
                task.modified.as_seconds(),
            ],
        )
        .expect("insert_task_from: failed to insert test fixture");
}

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
        uuid: "00000000-0000-0000-0000-000000000020".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("earlier entry").unwrap(),
        entry: Timestamp::new(500).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(500).unwrap(),
    };
    let first_by_id = Task {
        uuid: "00000000-0000-0000-0000-00000000001e".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("first by id").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let second_by_id = Task {
        uuid: "00000000-0000-0000-0000-00000000001f".parse().unwrap(),
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
        uuid: "00000000-0000-0000-0000-000000000021".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uuid: "00000000-0000-0000-0000-000000000022".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uuid: "00000000-0000-0000-0000-000000000023".parse().unwrap(),
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
    let filter = Filter::default().with_report_status(Status::Pending);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending]);
}

#[test]
fn list_tasks_filter_completed_only() {
    let db = setup();
    let pending = Task {
        uuid: "00000000-0000-0000-0000-000000000024".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uuid: "00000000-0000-0000-0000-000000000025".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uuid: "00000000-0000-0000-0000-000000000026".parse().unwrap(),
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
    let filter = Filter::default().with_report_status(Status::Completed);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![completed]);
}

#[test]
fn list_tasks_filter_deleted_only() {
    let db = setup();
    let pending = Task {
        uuid: "00000000-0000-0000-0000-000000000027".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uuid: "00000000-0000-0000-0000-000000000028".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let deleted = Task {
        uuid: "00000000-0000-0000-0000-000000000029".parse().unwrap(),
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
    let filter = Filter::default().with_report_status(Status::Deleted);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![deleted]);
}

// Filter by UUID

#[test]
fn list_tasks_filter_single_uuid() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-000000000037".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("target").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other1 = Task {
        uuid: "00000000-0000-0000-0000-000000000038".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("other 1").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let other2 = Task {
        uuid: "00000000-0000-0000-0000-000000000039".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("other 2").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &other1);
    insert_task_from(&db, &other2);
    let filter = Filter::default().with_uuids([UuidPrefix::from(
        "00000000-0000-0000-0000-000000000037"
            .parse::<Uuid>()
            .unwrap(),
    )]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_multiple_uuids() {
    let db = setup();
    let first = Task {
        uuid: "00000000-0000-0000-0000-00000000003a".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("first").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let second = Task {
        uuid: "00000000-0000-0000-0000-00000000003b".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("second").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uuid: "00000000-0000-0000-0000-00000000003c".parse().unwrap(),
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
    let filter = Filter::default().with_uuids([
        UuidPrefix::from(
            "00000000-0000-0000-0000-00000000003a"
                .parse::<Uuid>()
                .unwrap(),
        ),
        UuidPrefix::from(
            "00000000-0000-0000-0000-00000000003b"
                .parse::<Uuid>()
                .unwrap(),
        ),
    ]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![first, second]);
}

#[test]
fn list_tasks_filter_nonexistent_uuid() {
    let db = setup();
    let task = Task {
        uuid: "00000000-0000-0000-0000-000000000046".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("existing").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let nonexistent: Uuid = "00000000-0000-0000-0000-000000000047".parse().unwrap();
    let filter = Filter::default().with_uuids([UuidPrefix::from(nonexistent)]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_short_uuid_prefix() {
    let db = setup();
    let expected = Task {
        uuid: "550e8400-e29b-41d4-a716-446655440000".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("target").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let decoy = Task {
        uuid: "deadbeef-0000-0000-0000-000000000000".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("decoy").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &decoy);
    let filter = Filter::default().with_uuids([UuidPrefix::parse("550e8400").unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_full_uuid_and_prefix_mixed() {
    let db = setup();
    let by_full = Task {
        uuid: "11111111-2222-3333-4444-555555555555".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("matched by full uuid").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let by_prefix = Task {
        uuid: "aabbccdd-0000-0000-0000-000000000001".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("matched by prefix").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uuid: "ffffffff-0000-0000-0000-000000000002".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("excluded").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &by_full);
    insert_task_from(&db, &by_prefix);
    insert_task_from(&db, &excluded);
    let filter = Filter::default().with_uuids([
        UuidPrefix::from(
            "11111111-2222-3333-4444-555555555555"
                .parse::<Uuid>()
                .unwrap(),
        ),
        UuidPrefix::parse("aabbccdd").unwrap(),
    ]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![by_full, by_prefix]);
}

// Filter by Index

#[test]
fn list_tasks_filter_single_index() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-00000000006c".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("target").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uuid: "00000000-0000-0000-0000-00000000006d".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("other").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &other);
    let filter = Filter::default().with_indices([Index::new(1).unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_multiple_indices() {
    let db = setup();
    let first = Task {
        uuid: "00000000-0000-0000-0000-00000000006e".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("first").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let second = Task {
        uuid: "00000000-0000-0000-0000-00000000006f".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("second").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uuid: "00000000-0000-0000-0000-000000000070".parse().unwrap(),
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
fn list_tasks_filter_nonexistent_index() {
    let db = setup();
    let task = Task {
        uuid: "00000000-0000-0000-0000-000000000075".parse().unwrap(),
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

#[test]
fn list_tasks_filter_index_with_completed_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uuid: "00000000-0000-0000-0000-000000000071".parse().unwrap(),
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
            uuid: "00000000-0000-0000-0000-000000000072".parse().unwrap(),
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
        .with_report_status(Status::Completed);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

// Filter by Index Range

fn five_pending(base: u128) -> [Task; 5] {
    std::array::from_fn(|i| {
        let n = i + 1;
        let secs = (n as i64) * 1000;
        Task {
            uuid: Uuid::from_u128(base + n as u128),
            index: Some(Index::new(n).unwrap()),
            description: Description::new(&format!("task {n}")).unwrap(),
            entry: Timestamp::new(secs).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(secs).unwrap(),
        }
    })
}

#[test]
fn list_tasks_filter_index_range() {
    let db = setup();
    let [t1, t2, t3, t4, t5] = five_pending(0x30 << 8);
    for t in [&t1, &t2, &t3, &t4, &t5] {
        insert_task_from(&db, t);
    }
    let filter = Filter::default().with_index_ranges([IndexRange::new(
        Index::new(1).unwrap(),
        Index::new(3).unwrap(),
    )
    .unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![t1, t2, t3]);
}

#[test]
fn list_tasks_filter_index_range_oversized_upper_returns_existing() {
    let db = setup();
    let [t1, t2, t3, t4, t5] = five_pending(0x31 << 8);
    for t in [&t1, &t2, &t3, &t4, &t5] {
        insert_task_from(&db, t);
    }
    let filter = Filter::default().with_index_ranges([IndexRange::new(
        Index::new(1).unwrap(),
        Index::new(100).unwrap(),
    )
    .unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![t1, t2, t3, t4, t5]);
}

// Filter by Word

#[test]
fn list_tasks_filter_word_long_case_insensitive() {
    let db = setup();
    let task = Task {
        uuid: "00000000-0000-0000-0000-00000000005a".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("Buy MILK today").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_words(["milk"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![task]);
}

#[test]
fn list_tasks_filter_word_substring_match() {
    let db = setup();
    let task = Task {
        uuid: "00000000-0000-0000-0000-00000000005b".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("refactoring code").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_words(["factor"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![task]);
}

#[test]
fn list_tasks_filter_word_no_match_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uuid: "00000000-0000-0000-0000-00000000005c".parse().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("buy milk").unwrap(),
            entry: Timestamp::new(1000).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(1000).unwrap(),
        },
    );
    let filter = Filter::default().with_words(["xyz"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_word_short_match() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-000000000065".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("hi there").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uuid: "00000000-0000-0000-0000-000000000066".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("go home").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &other);
    let filter = Filter::default().with_words(["hi"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_word_short_case_insensitive() {
    let db = setup();
    let task = Task {
        uuid: "00000000-0000-0000-0000-000000000067".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("HI there").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    insert_task_from(&db, &task);
    let filter = Filter::default().with_words(["hi"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![task]);
}

#[test]
fn list_tasks_filter_multiple_words_all_match() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-00000000005d".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk and eggs").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let bread = Task {
        uuid: "00000000-0000-0000-0000-00000000005e".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy bread").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &bread);
    let filter = Filter::default().with_words(["milk", "eggs"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_multiple_words_partial_match_returns_empty() {
    let db = setup();
    insert_task_from(
        &db,
        &Task {
            uuid: "00000000-0000-0000-0000-00000000005f".parse().unwrap(),
            index: Some(Index::new(1).unwrap()),
            description: Description::new("buy milk").unwrap(),
            entry: Timestamp::new(1000).unwrap(),
            completed: None,
            deleted: None,
            modified: Timestamp::new(1000).unwrap(),
        },
    );
    let filter = Filter::default().with_words(["milk", "eggs"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert!(tasks.is_empty());
}

#[test]
fn list_tasks_filter_mixed_long_and_short_words() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-000000000060".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("hi there world").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let only_short = Task {
        uuid: "00000000-0000-0000-0000-000000000061".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("hi nothing").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let only_long = Task {
        uuid: "00000000-0000-0000-0000-000000000062".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("world only").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &only_short);
    insert_task_from(&db, &only_long);
    let filter = Filter::default().with_words(["hi", "world"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_word_korean_short() {
    let db = setup();
    let korean = Task {
        uuid: "00000000-0000-0000-0000-000000000055".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("한글 작업").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let english = Task {
        uuid: "00000000-0000-0000-0000-000000000056".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("english task").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &korean);
    insert_task_from(&db, &english);
    let filter = Filter::default().with_words(["한"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![korean]);
}

#[test]
fn list_tasks_filter_word_with_like_meta_character() {
    let db = setup();
    let percent = Task {
        uuid: "00000000-0000-0000-0000-000000000063".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("50% off").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let plain = Task {
        uuid: "00000000-0000-0000-0000-000000000064".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("abc off").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &percent);
    insert_task_from(&db, &plain);
    let filter = Filter::default().with_words(["%"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![percent]);
}

// Filter combinations — UUID

#[test]
fn list_tasks_filter_uuid_and_index() {
    let db = setup();
    let by_uuid = Task {
        uuid: "00000000-0000-0000-0000-000000000002".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("matched by uuid").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let by_index = Task {
        uuid: "00000000-0000-0000-0000-000000000003".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("matched by index").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let excluded = Task {
        uuid: "00000000-0000-0000-0000-000000000004".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("excluded").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &by_uuid);
    insert_task_from(&db, &by_index);
    insert_task_from(&db, &excluded);
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(
            "00000000-0000-0000-0000-000000000002"
                .parse::<Uuid>()
                .unwrap(),
        )])
        .with_indices([Index::new(2).unwrap()]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![by_uuid, by_index]);
}

#[test]
fn list_tasks_filter_uuid_with_status() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-000000000043".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("pending").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uuid: "00000000-0000-0000-0000-000000000044".parse().unwrap(),
        index: None,
        description: Description::new("completed").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: Some(Timestamp::new(3000).unwrap()),
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let other = Task {
        uuid: "00000000-0000-0000-0000-000000000045".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("other pending").unwrap(),
        entry: Timestamp::new(4000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(4000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &completed);
    insert_task_from(&db, &other);
    let filter = Filter::default()
        .with_uuids([
            UuidPrefix::from(
                "00000000-0000-0000-0000-000000000043"
                    .parse::<Uuid>()
                    .unwrap(),
            ),
            UuidPrefix::from(
                "00000000-0000-0000-0000-000000000044"
                    .parse::<Uuid>()
                    .unwrap(),
            ),
        ])
        .with_report_status(Status::Pending);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

// Filter combinations — Index

#[test]
fn list_tasks_filter_index_and_range() {
    let db = setup();
    let tasks = five_pending(0x32 << 8);
    for t in &tasks {
        insert_task_from(&db, t);
    }
    let [t1, t2, _, t4, _] = tasks;
    let filter = Filter::default()
        .with_indices([Index::new(4).unwrap()])
        .with_index_ranges([
            IndexRange::new(Index::new(1).unwrap(), Index::new(2).unwrap()).unwrap(),
        ]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![t1, t2, t4]);
}

// Filter combinations — Word

#[test]
fn list_tasks_filter_word_with_status() {
    let db = setup();
    let pending = Task {
        uuid: "00000000-0000-0000-0000-000000000068".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let completed = Task {
        uuid: "00000000-0000-0000-0000-000000000069".parse().unwrap(),
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
        .with_words(["milk"])
        .with_report_status(Status::Pending);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![pending]);
}

#[test]
fn list_tasks_filter_word_with_uuid_prefix() {
    let db = setup();
    let expected = Task {
        uuid: "aabbccdd-0000-0000-0000-00000000006a".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uuid: "00000000-0000-0000-0000-00000000006b".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy milk too").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &other);
    let filter = Filter::default()
        .with_uuids([UuidPrefix::parse("aabbccdd").unwrap()])
        .with_words(["milk"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_word_with_index() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-000000000048".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("buy milk").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other_word = Task {
        uuid: "00000000-0000-0000-0000-000000000049".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("buy bread").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    let other_index = Task {
        uuid: "00000000-0000-0000-0000-00000000004a".parse().unwrap(),
        index: Some(Index::new(3).unwrap()),
        description: Description::new("buy milk too").unwrap(),
        entry: Timestamp::new(3000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(3000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &other_word);
    insert_task_from(&db, &other_index);
    let filter = Filter::default()
        .with_indices([Index::new(1).unwrap()])
        .with_words(["milk"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}

#[test]
fn list_tasks_filter_word_with_index_short() {
    let db = setup();
    let expected = Task {
        uuid: "00000000-0000-0000-0000-00000000004b".parse().unwrap(),
        index: Some(Index::new(1).unwrap()),
        description: Description::new("hi there").unwrap(),
        entry: Timestamp::new(1000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(1000).unwrap(),
    };
    let other = Task {
        uuid: "00000000-0000-0000-0000-00000000004c".parse().unwrap(),
        index: Some(Index::new(2).unwrap()),
        description: Description::new("hi friend").unwrap(),
        entry: Timestamp::new(2000).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(2000).unwrap(),
    };
    insert_task_from(&db, &expected);
    insert_task_from(&db, &other);
    let filter = Filter::default()
        .with_indices([Index::new(1).unwrap()])
        .with_words(["hi"]);

    let tasks = db.list_tasks(&filter).unwrap();

    assert_eq!(tasks, vec![expected]);
}
