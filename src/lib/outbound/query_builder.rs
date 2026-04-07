use crate::domain::task::{Filter, Status, TaskModification, Timestamp, UniqueID};
use anyhow::Context;
use rusqlite::ToSql;

const ALL_STATUSES: usize = 3;

type Clause = (String, Vec<Box<dyn ToSql>>);

pub(crate) fn build_where_clause(filter: &Filter) -> anyhow::Result<Option<Clause>> {
    if filter.is_empty() {
        return Ok(None);
    }
    let mut clauses = Vec::new();
    let mut params = Vec::new();
    if let Some((id_clause, id_params)) = build_id_clause(filter)? {
        clauses.push(id_clause);
        params.extend(id_params);
    }
    if let Some(status_clause) = build_status_clause(filter) {
        clauses.push(status_clause);
    }
    if clauses.is_empty() {
        Ok(None)
    } else {
        Ok(Some((format!("WHERE {}", clauses.join(" AND ")), params)))
    }
}

fn build_id_clause(filter: &Filter) -> anyhow::Result<Option<Clause>> {
    let uids = filter.uids();
    let uid_clause = (!uids.is_empty()).then(|| {
        let params: Vec<Box<dyn ToSql>> = uids
            .iter()
            .map(|uid| Box::new(uid.to_string()) as Box<dyn ToSql>)
            .collect();
        (format!("t.id IN ({})", repeat_vars(uids.len())), params)
    });
    let indices = filter.indices();
    let index_clause = (!indices.is_empty())
        .then(|| -> anyhow::Result<Clause> {
            let params: Vec<Box<dyn ToSql>> = indices
                .iter()
                .map(|index| i64::try_from(index.get()).map(|v| Box::new(v) as Box<dyn ToSql>))
                .collect::<Result<_, _>>()
                .context("task index exceeds i64 range")?;
            Ok((
                format!("tpr.row_id IN ({})", repeat_vars(indices.len())),
                params,
            ))
        })
        .transpose()?;
    Ok(match (uid_clause, index_clause) {
        (None, None) => None,
        (Some(single), None) | (None, Some(single)) => Some(single),
        (Some((uid_cl, uid_params)), Some((idx_cl, idx_params))) => Some((
            format!("({uid_cl} OR {idx_cl})"),
            uid_params.into_iter().chain(idx_params).collect(),
        )),
    })
}

fn build_status_clause(filter: &Filter) -> Option<String> {
    let statuses = filter.statuses();
    if statuses.is_empty() || statuses.len() == ALL_STATUSES {
        return None;
    }
    let mut conditions = Vec::new();
    for status in statuses {
        match status {
            Status::Pending => conditions.push("(t.deleted IS NULL AND t.completed IS NULL)"),
            Status::Completed => conditions.push("(t.deleted IS NULL AND t.completed IS NOT NULL)"),
            Status::Deleted => conditions.push("(t.deleted IS NOT NULL)"),
        }
    }
    let joined = conditions.join(" OR ");
    if conditions.len() > 1 {
        Some(format!("({joined})"))
    } else {
        Some(joined)
    }
}

pub(crate) fn build_update_clause(
    modification: TaskModification,
    targets: &[&UniqueID],
) -> anyhow::Result<Clause> {
    if modification.is_empty() {
        return Err(anyhow::anyhow!("No modification specified"));
    }
    if targets.is_empty() {
        return Err(anyhow::anyhow!("No target specified"));
    }

    // Collect update expressions and their parameters based on the provided modification
    let (updates, update_params): (Vec<String>, Vec<Box<dyn ToSql>>) = [
        modification.description.map(|desc| {
            (
                "description = ?".to_string(),
                Box::new(desc.to_string()) as Box<dyn ToSql>,
            )
        }),
        modification
            .completed
            .map(|c| timestamp_set_entry("completed", c)),
        modification
            .deleted
            .map(|d| timestamp_set_entry("deleted", d)),
    ]
    .into_iter()
    .flatten()
    .unzip();

    // Combine update values parameters with target IDs
    let params: Vec<Box<dyn ToSql>> = update_params
        .into_iter()
        // collect target ID as SQL parameters
        .chain(
            targets
                .iter()
                .map(|uid| Box::new(uid.to_string()) as Box<dyn ToSql>)
                .collect::<Vec<Box<dyn ToSql>>>(),
        )
        .collect();

    // Build the final SQL clause with placeholders for updates and target IDs
    let clause = format!(
        "UPDATE task SET {} WHERE id IN ({})",
        updates.join(", "),
        repeat_vars(targets.len())
    );
    Ok((clause, params))
}

fn timestamp_set_entry(column: &str, value: Option<Timestamp>) -> (String, Box<dyn ToSql>) {
    let sql_value = value.map(|t| t.get());
    (
        format!("{column} = ?"),
        Box::new(sql_value) as Box<dyn ToSql>,
    )
}

// ref: https://docs.rs/rusqlite/latest/rusqlite/struct.ParamsFromIter.html#realistic-use-case
fn repeat_vars(count: usize) -> String {
    std::iter::repeat_n("?", count)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
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
        let filter =
            Filter::new().with_statuses([Status::Pending, Status::Completed, Status::Deleted]);

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

        assert!(build_update_clause(modification, &[&uid]).is_err());
    }

    #[test]
    fn build_update_clause_with_empty_targets() {
        use crate::domain::task::Description;

        let modification = TaskModification {
            description: Some(Description::new("updated").unwrap()),
            completed: None,
            deleted: None,
        };

        assert!(build_update_clause(modification, &[]).is_err());
    }

    #[test]
    fn build_update_clause_with_description() {
        use crate::domain::task::{Description, UniqueID};

        let uid = UniqueID::new();
        let modification = TaskModification {
            description: Some(Description::new("updated").unwrap()),
            completed: None,
            deleted: None,
        };

        let (clause, params) = build_update_clause(modification, &[&uid]).unwrap();
        assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?)");
        assert_eq!(params.len(), 2);
        assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
        assert_eq!(to_value(params[1].as_ref()), Value::Text(uid.to_string()));
    }

    #[test]
    fn build_update_clause_with_completed_set() {
        use crate::domain::task::{Timestamp, UniqueID};

        let uid = UniqueID::new();
        let modification = TaskModification {
            description: None,
            completed: Some(Some(Timestamp::new(1700000000).unwrap())),
            deleted: None,
        };

        let (clause, params) = build_update_clause(modification, &[&uid]).unwrap();
        assert_eq!(clause, "UPDATE task SET completed = ? WHERE id IN (?)");
        assert_eq!(params.len(), 2);
        assert_eq!(to_value(params[0].as_ref()), Value::Integer(1700000000));
        assert_eq!(to_value(params[1].as_ref()), Value::Text(uid.to_string()));
    }

    #[test]
    fn build_update_clause_with_completed_cleared() {
        use crate::domain::task::UniqueID;

        let uid = UniqueID::new();
        let modification = TaskModification {
            description: None,
            completed: Some(None),
            deleted: None,
        };

        let (clause, params) = build_update_clause(modification, &[&uid]).unwrap();
        assert_eq!(clause, "UPDATE task SET completed = ? WHERE id IN (?)");
        assert_eq!(params.len(), 2);
        assert_eq!(to_value(params[0].as_ref()), Value::Null);
        assert_eq!(to_value(params[1].as_ref()), Value::Text(uid.to_string()));
    }

    #[test]
    fn build_update_clause_with_deleted_set() {
        use crate::domain::task::{Timestamp, UniqueID};

        let uid = UniqueID::new();
        let modification = TaskModification {
            description: None,
            completed: None,
            deleted: Some(Some(Timestamp::new(1700000000).unwrap())),
        };

        let (clause, params) = build_update_clause(modification, &[&uid]).unwrap();
        assert_eq!(clause, "UPDATE task SET deleted = ? WHERE id IN (?)");
        assert_eq!(params.len(), 2);
        assert_eq!(to_value(params[0].as_ref()), Value::Integer(1700000000));
        assert_eq!(to_value(params[1].as_ref()), Value::Text(uid.to_string()));
    }

    #[test]
    fn build_update_clause_with_deleted_cleared() {
        use crate::domain::task::UniqueID;

        let uid = UniqueID::new();
        let modification = TaskModification {
            description: None,
            completed: None,
            deleted: Some(None),
        };

        let (clause, params) = build_update_clause(modification, &[&uid]).unwrap();
        assert_eq!(clause, "UPDATE task SET deleted = ? WHERE id IN (?)");
        assert_eq!(params.len(), 2);
        assert_eq!(to_value(params[0].as_ref()), Value::Null);
        assert_eq!(to_value(params[1].as_ref()), Value::Text(uid.to_string()));
    }

    #[test]
    fn build_update_clause_with_multiple_fields() {
        use crate::domain::task::{Description, Timestamp, UniqueID};

        let uid = UniqueID::new();
        let modification = TaskModification {
            description: Some(Description::new("updated").unwrap()),
            completed: Some(Some(Timestamp::new(1700000000).unwrap())),
            deleted: None,
        };

        let (clause, params) = build_update_clause(modification, &[&uid]).unwrap();
        assert_eq!(
            clause,
            "UPDATE task SET description = ?, completed = ? WHERE id IN (?)"
        );
        assert_eq!(params.len(), 3);
        assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
        assert_eq!(to_value(params[1].as_ref()), Value::Integer(1700000000));
        assert_eq!(to_value(params[2].as_ref()), Value::Text(uid.to_string()));
    }

    #[test]
    fn build_update_clause_with_multiple_targets() {
        use crate::domain::task::{Description, UniqueID};

        let uid1 = UniqueID::new();
        let uid2 = UniqueID::new();
        let modification = TaskModification {
            description: Some(Description::new("updated").unwrap()),
            completed: None,
            deleted: None,
        };

        let (clause, params) = build_update_clause(modification, &[&uid1, &uid2]).unwrap();
        assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?,?)");
        assert_eq!(params.len(), 3);
        assert_eq!(to_value(params[0].as_ref()), Value::Text("updated".into()));
        assert_eq!(to_value(params[1].as_ref()), Value::Text(uid1.to_string()));
        assert_eq!(to_value(params[2].as_ref()), Value::Text(uid2.to_string()));
    }
}
