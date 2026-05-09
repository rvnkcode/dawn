use rusqlite::types::{ToSqlOutput, Value, ValueRef};

use super::*;
use crate::domain::task::UuidPrefix;

// Convert ToSql parameter into a Value for easier assertions in tests
fn to_value(param: &dyn ToSql) -> Value {
    match param.to_sql().unwrap() {
        ToSqlOutput::Owned(value) => value,
        ToSqlOutput::Borrowed(ValueRef::Text(s)) => {
            Value::Text(std::str::from_utf8(s).unwrap().to_string())
        }
        ToSqlOutput::Borrowed(ValueRef::Integer(i)) => Value::Integer(i),
        ToSqlOutput::Borrowed(ValueRef::Null) => Value::Null,
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn repeat_vars_single() {
    assert_eq!(repeat_vars(1), "?");
}

#[test]
fn repeat_vars_multiple() {
    assert_eq!(repeat_vars(3), "?,?,?");
}

#[test]
fn build_where_clause_with_empty_filter() {
    let filter = Filter::default();

    assert!(build_where_clause(&filter).unwrap().is_none());
}

// filter.uuids

#[test]
fn build_where_clause_with_single_uuid() {
    use uuid::Uuid;

    let uuid_str = Uuid::new_v4().to_string();
    let filter = Filter::default().with_uuids([UuidPrefix::parse(&uuid_str).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE t.id IN (?)");
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_where_clause_with_multiple_uids() {
    use uuid::Uuid;

    let uuid1_str = Uuid::new_v4().to_string();
    let uuid2_str = Uuid::new_v4().to_string();
    let filter = Filter::default().with_uuids([
        UuidPrefix::parse(&uuid1_str).unwrap(),
        UuidPrefix::parse(&uuid2_str).unwrap(),
    ]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE t.id IN (?,?)");
    assert_eq!(params.len(), 2);
    let values: Vec<Value> = params.iter().map(|p| to_value(p.as_ref())).collect();
    assert!(values.contains(&Value::Text(uuid1_str)));
    assert!(values.contains(&Value::Text(uuid2_str)));
}

#[test]
fn build_where_clause_with_short_uuid_prefix() {
    let filter = Filter::default().with_uuids([UuidPrefix::parse("abc12345").unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE t.id LIKE ?");
    assert_eq!(
        to_value(params[0].as_ref()),
        Value::Text("abc12345%".into())
    );
}

#[test]
fn build_where_clause_mixes_full_uuid_and_short_prefix() {
    use uuid::Uuid;

    let full_uuid_str = Uuid::new_v4().to_string();
    let filter = Filter::default().with_uuids([
        UuidPrefix::parse(&full_uuid_str).unwrap(),
        UuidPrefix::parse("abc12345").unwrap(),
    ]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE (t.id IN (?) OR t.id LIKE ?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(full_uuid_str));
    assert_eq!(
        to_value(params[1].as_ref()),
        Value::Text("abc12345%".into())
    );
}

// filter.indices

#[test]
fn build_where_clause_with_single_index() {
    use crate::domain::task::Index;

    let filter = Filter::default().with_indices([Index::new(1).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE tpr.row_id IN (?)");
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1));
}

#[test]
fn build_where_clause_with_multiple_indices() {
    use crate::domain::task::Index;

    let filter = Filter::default().with_indices([Index::new(1).unwrap(), Index::new(2).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE tpr.row_id IN (?,?)");
    assert_eq!(params.len(), 2);
    let values: Vec<Value> = params.iter().map(|p| to_value(p.as_ref())).collect();
    assert!(values.contains(&Value::Integer(1)));
    assert!(values.contains(&Value::Integer(2)));
}

// filter.index_ranges

#[test]
fn build_where_clause_with_single_index_range() {
    use crate::domain::task::{Index, IndexRange};

    let filter = Filter::default().with_index_ranges([IndexRange::new(
        Index::new(1).unwrap(),
        Index::new(3).unwrap(),
    )
    .unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE tpr.row_id BETWEEN ? AND ?");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(3));
}

#[test]
fn build_where_clause_with_multiple_index_ranges() {
    use crate::domain::task::{Index, IndexRange};

    let filter = Filter::default().with_index_ranges([
        IndexRange::new(Index::new(1).unwrap(), Index::new(3).unwrap()).unwrap(),
        IndexRange::new(Index::new(5).unwrap(), Index::new(7).unwrap()).unwrap(),
    ]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(
        clause,
        "WHERE (tpr.row_id BETWEEN ? AND ? OR tpr.row_id BETWEEN ? AND ?)"
    );
    assert_eq!(params.len(), 4);
    let values: Vec<i64> = params
        .iter()
        .map(|p| match to_value(p.as_ref()) {
            Value::Integer(n) => n,
            v => panic!("expected integer parameter, got {v:?}"),
        })
        .collect();
    // HashSet ordering is non-deterministic, but each (start, end) pair must stay together
    let mut pairs = [(values[0], values[1]), (values[2], values[3])];
    pairs.sort();
    assert_eq!(pairs, [(1, 3), (5, 7)]);
}

// filter.report_status

#[test]
fn build_where_clause_with_pending() {
    let filter = Filter::default().with_report_status(Status::Pending);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE (t.deleted IS NULL AND t.completed IS NULL)");
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_with_completed() {
    let filter = Filter::default().with_report_status(Status::Completed);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(
        clause,
        "WHERE (t.deleted IS NULL AND t.completed IS NOT NULL)"
    );
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_with_deleted() {
    let filter = Filter::default().with_report_status(Status::Deleted);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE t.deleted IS NOT NULL");
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_without_report_status_omits_status_clause() {
    use crate::domain::task::Index;

    let filter = Filter::default().with_indices([Index::new(1).unwrap()]);

    let (clause, _) = build_where_clause(&filter).unwrap().unwrap();

    assert!(!clause.contains("completed"));
    assert!(!clause.contains("deleted"));
}

// filter.words

#[test]
fn build_where_clause_with_single_long_word() {
    let filter = Filter::default().with_words(["hello"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(
        clause,
        "WHERE t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)"
    );
    assert_eq!(params.len(), 1);
    assert_eq!(
        to_value(params[0].as_ref()),
        Value::Text("\"hello\"".into())
    );
}

#[test]
fn build_where_clause_with_multiple_long_words() {
    let filter = Filter::default().with_words(["hello", "world"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(
        clause,
        "WHERE t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)"
    );
    assert_eq!(params.len(), 1);
    assert_eq!(
        to_value(params[0].as_ref()),
        Value::Text("\"hello\" AND \"world\"".into())
    );
}

#[test]
fn build_where_clause_with_single_short_word() {
    let filter = Filter::default().with_words(["hi"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, r"WHERE t.description LIKE ? ESCAPE '\'");
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("%hi%".into()));
}

#[test]
fn build_where_clause_counts_chars_not_bytes_for_korean_short() {
    let filter = Filter::default().with_words(["한"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, r"WHERE t.description LIKE ? ESCAPE '\'");
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("%한%".into()));
}

#[test]
fn build_where_clause_with_multiple_short_words() {
    let filter = Filter::default().with_words(["a", "bb"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(
        clause,
        r"WHERE (t.description LIKE ? ESCAPE '\' AND t.description LIKE ? ESCAPE '\')"
    );
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("%a%".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Text("%bb%".into()));
}

#[test]
fn build_where_clause_with_mixed_word_lengths() {
    let filter = Filter::default().with_words(["hi", "hello"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(
        clause,
        r"WHERE (t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?) AND t.description LIKE ? ESCAPE '\')"
    );
    assert_eq!(params.len(), 2);
    assert_eq!(
        to_value(params[0].as_ref()),
        Value::Text("\"hello\"".into())
    );
    assert_eq!(to_value(params[1].as_ref()), Value::Text("%hi%".into()));
}

// build ID clauses

#[test]
fn build_where_clause_with_uuid_and_index() {
    use uuid::Uuid;

    use crate::domain::task::Index;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(uuid)])
        .with_indices([Index::new(1).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE (t.id IN (?) OR tpr.row_id IN (?))");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1));
}

#[test]
fn build_where_clause_with_uuid_and_index_range() {
    use uuid::Uuid;

    use crate::domain::task::{Index, IndexRange};

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(uuid)])
        .with_index_ranges([
            IndexRange::new(Index::new(1).unwrap(), Index::new(3).unwrap()).unwrap(),
        ]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(clause, "WHERE (t.id IN (?) OR tpr.row_id BETWEEN ? AND ?)");
    assert_eq!(params.len(), 3);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1));
    assert_eq!(to_value(params[2].as_ref()), Value::Integer(3));
}

#[test]
fn build_where_clause_with_index_and_index_range() {
    use crate::domain::task::{Index, IndexRange};

    let filter = Filter::default()
        .with_indices([Index::new(7).unwrap()])
        .with_index_ranges([
            IndexRange::new(Index::new(1).unwrap(), Index::new(3).unwrap()).unwrap(),
        ]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(
        clause,
        "WHERE (tpr.row_id IN (?) OR tpr.row_id BETWEEN ? AND ?)"
    );
    assert_eq!(params.len(), 3);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(7));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1));
    assert_eq!(to_value(params[2].as_ref()), Value::Integer(3));
}

#[test]
fn build_where_clause_with_uuid_and_index_and_index_range() {
    use uuid::Uuid;

    use crate::domain::task::{Index, IndexRange};

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(uuid)])
        .with_indices([Index::new(7).unwrap()])
        .with_index_ranges([
            IndexRange::new(Index::new(1).unwrap(), Index::new(3).unwrap()).unwrap(),
        ]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(
        clause,
        "WHERE (t.id IN (?) OR tpr.row_id IN (?) OR tpr.row_id BETWEEN ? AND ?)"
    );
    assert_eq!(params.len(), 4);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(7));
    assert_eq!(to_value(params[2].as_ref()), Value::Integer(1));
    assert_eq!(to_value(params[3].as_ref()), Value::Integer(3));
}

// filter IDs + status

#[test]
fn build_where_clause_with_uuid_and_status() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(uuid)])
        .with_report_status(Status::Pending);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert!(clause.starts_with("WHERE "));
    assert!(clause.contains("t.id IN (?)"));
    assert!(clause.contains(" AND "));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_where_clause_with_status_and_words() {
    let filter = Filter::default()
        .with_report_status(Status::Pending)
        .with_words(["hello"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert!(clause.starts_with("WHERE "));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
    assert!(clause.contains("t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)"));
    assert!(clause.contains(" AND "));
    assert_eq!(params.len(), 1);
    assert_eq!(
        to_value(params[0].as_ref()),
        Value::Text("\"hello\"".into())
    );
}

#[test]
fn build_where_clause_with_uuid_and_words() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(uuid)])
        .with_words(["hi"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert_eq!(
        clause,
        r"WHERE t.id IN (?) AND t.description LIKE ? ESCAPE '\'"
    );
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
    assert_eq!(to_value(params[1].as_ref()), Value::Text("%hi%".into()));
}

#[test]
fn build_where_clause_with_uuid_and_status_and_words() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let filter = Filter::default()
        .with_uuids([UuidPrefix::from(uuid)])
        .with_report_status(Status::Pending)
        .with_words(["hello"]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();

    assert!(clause.starts_with("WHERE "));
    assert!(clause.contains("t.id IN (?)"));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
    assert!(clause.contains("t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)"));
    assert!(clause.contains(" AND "));
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uuid_str));
    assert_eq!(
        to_value(params[1].as_ref()),
        Value::Text("\"hello\"".into())
    );
}

// escape_fts5_term

#[test]
fn escape_fts5_term_wraps_plain_string_in_quotes() {
    assert_eq!(escape_fts5_term(""), "\"\"");
    assert_eq!(escape_fts5_term("hello"), "\"hello\"");
    assert_eq!(escape_fts5_term("한글"), "\"한글\"");
}

#[test]
fn escape_fts5_term_doubles_internal_quotes() {
    assert_eq!(escape_fts5_term("a\"b"), "\"a\"\"b\"");
    assert_eq!(escape_fts5_term("\""), "\"\"\"\"");
    assert_eq!(escape_fts5_term("a\"b\"c"), "\"a\"\"b\"\"c\"");
}

// escape_like

#[test]
fn escape_like_passthrough_when_no_metachars() {
    assert_eq!(escape_like(""), "");
    assert_eq!(escape_like("abc"), "abc");
    assert_eq!(escape_like("한글"), "한글");
}

#[test]
fn escape_like_escapes_percent_underscore_and_backslash() {
    assert_eq!(escape_like("%"), r"\%");
    assert_eq!(escape_like("_"), r"\_");
    assert_eq!(escape_like("\\"), r"\\");
    assert_eq!(escape_like("a%b_c\\d"), r"a\%b\_c\\d");
}

// build_update_clause

#[test]
fn build_update_clause_with_empty_modification() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: None,
    };

    assert!(build_update_clause(&modification, &[uuid]).is_err());
}

#[test]
fn build_update_clause_with_empty_targets() {
    use crate::domain::task::Description;

    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: None,
        deleted: None,
    };

    assert!(build_update_clause(&modification, &[]).is_err());
}

#[test]
fn build_update_clause_with_description() {
    use uuid::Uuid;

    use crate::domain::task::Description;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: None,
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uuid]).unwrap();
    assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_update_clause_with_completed_set() {
    use uuid::Uuid;

    use crate::domain::task::Timestamp;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let modification = TaskModification {
        description: None,
        completed: Some(Some(Timestamp::new(1700000000).unwrap())),
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uuid]).unwrap();
    assert_eq!(
        clause,
        "UPDATE task SET completed = IFNULL(completed, ?) WHERE id IN (?)"
    );
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1700000000));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_update_clause_with_completed_cleared() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let modification = TaskModification {
        description: None,
        completed: Some(None),
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uuid]).unwrap();
    assert_eq!(clause, "UPDATE task SET completed = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Null);
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_update_clause_with_deleted_set() {
    use uuid::Uuid;

    use crate::domain::task::Timestamp;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
    };

    let (clause, params) = build_update_clause(&modification, &[uuid]).unwrap();
    assert_eq!(
        clause,
        "UPDATE task SET deleted = IFNULL(deleted, ?) WHERE id IN (?)"
    );
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1700000000));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_update_clause_with_deleted_cleared() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(None),
    };

    let (clause, params) = build_update_clause(&modification, &[uuid]).unwrap();
    assert_eq!(clause, "UPDATE task SET deleted = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Null);
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_update_clause_with_multiple_fields() {
    use uuid::Uuid;

    use crate::domain::task::{Description, Timestamp};

    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: Some(Some(Timestamp::new(1700000000).unwrap())),
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uuid]).unwrap();
    assert_eq!(
        clause,
        "UPDATE task SET description = ?, completed = IFNULL(completed, ?) WHERE id IN (?)"
    );
    assert_eq!(params.len(), 3);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1700000000));
    assert_eq!(to_value(params[2].as_ref()), Value::Text(uuid_str));
}

#[test]
fn build_update_clause_with_multiple_targets() {
    use uuid::Uuid;

    use crate::domain::task::Description;

    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let uuid1_str = uuid1.to_string();
    let uuid2_str = uuid2.to_string();
    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: None,
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uuid1, uuid2]).unwrap();
    assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?,?)");
    assert_eq!(params.len(), 3);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uuid1_str));
    assert_eq!(to_value(params[2].as_ref()), Value::Text(uuid2_str));
}

// build_order_clause

#[test]
fn build_order_clause_empty_returns_default() {
    let filter = Filter::default();
    assert_eq!(build_order_clause(&filter), "ORDER BY t.entry, t.id");
}

#[test]
fn build_order_clause_completed_asc_appends_id_tiebreaker() {
    let filter = Filter::default().with_sort_key(SortKey::Completed(Direction::Asc));
    assert_eq!(
        build_order_clause(&filter),
        "ORDER BY t.completed ASC, t.id ASC",
    );
}

#[test]
fn build_order_clause_entry_desc_appends_id_tiebreaker() {
    let filter = Filter::default().with_sort_key(SortKey::Entry(Direction::Desc));
    assert_eq!(
        build_order_clause(&filter),
        "ORDER BY t.entry DESC, t.id ASC",
    );
}
