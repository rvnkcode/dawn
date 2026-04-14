use super::*;
use rusqlite::types::{ToSqlOutput, Value, ValueRef};

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
fn build_where_clause_with_empty_filter() {
    let filter = Filter::new();

    assert!(build_where_clause(&filter).unwrap().is_none());
}

// filter.statuses

#[test]
fn build_where_clause_with_pending_only() {
    let filter = Filter::new().with_statuses([Status::Pending]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE (t.deleted IS NULL AND t.completed IS NULL)");
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_with_completed_only() {
    let filter = Filter::new().with_statuses([Status::Completed]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(
        clause,
        "WHERE (t.deleted IS NULL AND t.completed IS NOT NULL)"
    );
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_with_deleted_only() {
    let filter = Filter::new().with_statuses([Status::Deleted]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE (t.deleted IS NOT NULL)");
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_with_two_statuses() {
    let filter = Filter::new().with_statuses([Status::Pending, Status::Completed]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert!(clause.starts_with("WHERE "));
    assert!(clause.contains(" OR "));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NOT NULL)"));
    assert!(params.is_empty());
}

#[test]
fn build_where_clause_with_all_statuses() {
    let filter = Filter::new().with_statuses([Status::Pending, Status::Completed, Status::Deleted]);

    assert!(build_where_clause(&filter).unwrap().is_none());
}

#[test]
fn repeat_vars_single() {
    assert_eq!(repeat_vars(1), "?");
}

#[test]
fn repeat_vars_multiple() {
    assert_eq!(repeat_vars(3), "?,?,?");
}

// filter.uids

#[test]
fn build_where_clause_with_single_uid() {
    use crate::domain::task::UniqueID;

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let filter = Filter::new().with_uids([uid]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE t.id IN (?)");
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_where_clause_with_multiple_uids() {
    use crate::domain::task::UniqueID;

    let uid1 = UniqueID::new();
    let uid2 = UniqueID::new();
    let uid1_str = uid1.to_string();
    let uid2_str = uid2.to_string();
    let filter = Filter::new().with_uids([uid1, uid2]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE t.id IN (?,?)");
    assert_eq!(params.len(), 2);
    let values: Vec<Value> = params.iter().map(|p| to_value(p.as_ref())).collect();
    assert!(values.contains(&Value::Text(uid1_str)));
    assert!(values.contains(&Value::Text(uid2_str)));
}

// filter.indices

#[test]
fn build_where_clause_with_single_index() {
    use crate::domain::task::Index;

    let filter = Filter::new().with_indices([Index::new(1).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE tpr.row_id IN (?)");
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1));
}

#[test]
fn build_where_clause_with_multiple_indices() {
    use crate::domain::task::Index;

    let filter = Filter::new().with_indices([Index::new(1).unwrap(), Index::new(2).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE tpr.row_id IN (?,?)");
    assert_eq!(params.len(), 2);
    let values: Vec<Value> = params.iter().map(|p| to_value(p.as_ref())).collect();
    assert!(values.contains(&Value::Integer(1)));
    assert!(values.contains(&Value::Integer(2)));
}

// filter.uids + filter.indices

#[test]
fn build_where_clause_with_uid_and_index() {
    use crate::domain::task::{Index, UniqueID};

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let filter = Filter::new()
        .with_uids([uid])
        .with_indices([Index::new(1).unwrap()]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert_eq!(clause, "WHERE (t.id IN (?) OR tpr.row_id IN (?))");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uid_str));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1));
}

#[test]
fn build_where_clause_with_uid_and_index_and_status() {
    use crate::domain::task::{Index, UniqueID};

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let filter = Filter::new()
        .with_uids([uid])
        .with_indices([Index::new(1).unwrap()])
        .with_statuses([Status::Pending]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert!(clause.starts_with("WHERE "));
    assert!(clause.contains("(t.id IN (?) OR tpr.row_id IN (?))"));
    assert!(clause.contains(" AND "));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uid_str));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1));
}

// filter.uids + filter.statuses

#[test]
fn build_where_clause_with_uid_and_status() {
    use crate::domain::task::UniqueID;

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let filter = Filter::new()
        .with_uids([uid])
        .with_statuses([Status::Pending]);

    let (clause, params) = build_where_clause(&filter).unwrap().unwrap();
    assert!(clause.starts_with("WHERE "));
    assert!(clause.contains("t.id IN (?)"));
    assert!(clause.contains(" AND "));
    assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
    assert_eq!(params.len(), 1);
    assert_eq!(to_value(params[0].as_ref()), Value::Text(uid_str));
}

// build_update_clause

#[test]
fn build_update_clause_with_empty_modification() {
    use crate::domain::task::UniqueID;

    let uid = UniqueID::new();
    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: None,
    };

    assert!(build_update_clause(&modification, &[uid]).is_err());
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
    use crate::domain::task::{Description, UniqueID};

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: None,
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uid]).unwrap();
    assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_update_clause_with_completed_set() {
    use crate::domain::task::{Timestamp, UniqueID};

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let modification = TaskModification {
        description: None,
        completed: Some(Some(Timestamp::new(1700000000).unwrap())),
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uid]).unwrap();
    assert_eq!(clause, "UPDATE task SET completed = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1700000000));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_update_clause_with_completed_cleared() {
    use crate::domain::task::UniqueID;

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let modification = TaskModification {
        description: None,
        completed: Some(None),
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uid]).unwrap();
    assert_eq!(clause, "UPDATE task SET completed = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Null);
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_update_clause_with_deleted_set() {
    use crate::domain::task::{Timestamp, UniqueID};

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
    };

    let (clause, params) = build_update_clause(&modification, &[uid]).unwrap();
    assert_eq!(clause, "UPDATE task SET deleted = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Integer(1700000000));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_update_clause_with_deleted_cleared() {
    use crate::domain::task::UniqueID;

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let modification = TaskModification {
        description: None,
        completed: None,
        deleted: Some(None),
    };

    let (clause, params) = build_update_clause(&modification, &[uid]).unwrap();
    assert_eq!(clause, "UPDATE task SET deleted = ? WHERE id IN (?)");
    assert_eq!(params.len(), 2);
    assert_eq!(to_value(params[0].as_ref()), Value::Null);
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_update_clause_with_multiple_fields() {
    use crate::domain::task::{Description, Timestamp, UniqueID};

    let uid = UniqueID::new();
    let uid_str = uid.to_string();
    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: Some(Some(Timestamp::new(1700000000).unwrap())),
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uid]).unwrap();
    assert_eq!(
        clause,
        "UPDATE task SET description = ?, completed = ? WHERE id IN (?)"
    );
    assert_eq!(params.len(), 3);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Integer(1700000000));
    assert_eq!(to_value(params[2].as_ref()), Value::Text(uid_str));
}

#[test]
fn build_update_clause_with_multiple_targets() {
    use crate::domain::task::{Description, UniqueID};

    let uid1 = UniqueID::new();
    let uid2 = UniqueID::new();
    let uid1_str = uid1.to_string();
    let uid2_str = uid2.to_string();
    let modification = TaskModification {
        description: Some(Description::new("updated").unwrap()),
        completed: None,
        deleted: None,
    };

    let (clause, params) = build_update_clause(&modification, &[uid1, uid2]).unwrap();
    assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?,?)");
    assert_eq!(params.len(), 3);
    assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
    assert_eq!(to_value(params[1].as_ref()), Value::Text(uid1_str));
    assert_eq!(to_value(params[2].as_ref()), Value::Text(uid2_str));
}
