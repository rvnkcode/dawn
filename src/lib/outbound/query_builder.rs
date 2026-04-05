use crate::domain::task::{Filter, Status};
use rusqlite::ToSql;

const ALL_STATUSES: usize = 3;

pub(crate) fn build_where_clause(filter: &Filter) -> Option<(String, Vec<Box<dyn ToSql>>)> {
    if filter.is_empty() {
        return None;
    }
    let mut clauses = Vec::new();
    let mut params = Vec::new();
    if let Some((id_clause, id_params)) = build_id_clause(filter) {
        clauses.push(id_clause);
        params.extend(id_params);
    }
    if let Some(status_clause) = build_status_clause(filter) {
        clauses.push(status_clause);
    }
    if clauses.is_empty() {
        None
    } else {
        Some((format!("WHERE {}", clauses.join(" AND ")), params))
    }
}

fn build_id_clause(filter: &Filter) -> Option<(String, Vec<Box<dyn ToSql>>)> {
    let uids = filter.uids();
    let uid_clause = (!uids.is_empty()).then(|| {
        let params: Vec<Box<dyn ToSql>> = uids
            .iter()
            .map(|uid| Box::new(uid.to_string()) as Box<dyn ToSql>)
            .collect();
        (format!("t.id IN ({})", repeat_vars(uids.len())), params)
    });
    let indices = filter.indices();
    let index_clause = (!indices.is_empty()).then(|| {
        let params: Vec<Box<dyn ToSql>> = indices
            .iter()
            .map(|index| Box::new(index.get() as i64) as Box<dyn ToSql>)
            .collect();
        (
            format!("tpr.row_id IN ({})", repeat_vars(indices.len())),
            params,
        )
    });
    match (uid_clause, index_clause) {
        (None, None) => None,
        (Some(single), None) | (None, Some(single)) => Some(single),
        (Some((uid_cl, uid_params)), Some((idx_cl, idx_params))) => Some((
            format!("({uid_cl} OR {idx_cl})"),
            uid_params.into_iter().chain(idx_params).collect(),
        )),
    }
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

// ref: https://docs.rs/rusqlite/latest/rusqlite/struct.ParamsFromIter.html#realistic-use-case
fn repeat_vars(count: usize) -> String {
    std::iter::repeat_n("?", count)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_where_clause_with_empty_filter() {
        let filter = Filter::new();

        assert!(build_where_clause(&filter).is_none());
    }

    // filter.statuses

    #[test]
    fn build_where_clause_with_pending_only() {
        let filter = Filter::new().with_statuses([Status::Pending]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE (t.deleted IS NULL AND t.completed IS NULL)");
        assert!(params.is_empty());
    }

    #[test]
    fn build_where_clause_with_completed_only() {
        let filter = Filter::new().with_statuses([Status::Completed]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(
            clause,
            "WHERE (t.deleted IS NULL AND t.completed IS NOT NULL)"
        );
        assert!(params.is_empty());
    }

    #[test]
    fn build_where_clause_with_deleted_only() {
        let filter = Filter::new().with_statuses([Status::Deleted]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE (t.deleted IS NOT NULL)");
        assert!(params.is_empty());
    }

    #[test]
    fn build_where_clause_with_two_statuses() {
        let filter = Filter::new().with_statuses([Status::Pending, Status::Completed]);

        let (clause, params) = build_where_clause(&filter).unwrap();
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

        assert!(build_where_clause(&filter).is_none());
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
        let filter = Filter::new().with_uids([uid]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE t.id IN (?)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn build_where_clause_with_multiple_uids() {
        use crate::domain::task::UniqueID;

        let uid1 = UniqueID::new();
        let uid2 = UniqueID::new();
        let filter = Filter::new().with_uids([uid1, uid2]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE t.id IN (?,?)");
        assert_eq!(params.len(), 2);
    }

    // filter.indices

    #[test]
    fn build_where_clause_with_single_index() {
        use crate::domain::task::Index;

        let filter = Filter::new().with_indices([Index::new(1).unwrap()]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE tpr.row_id IN (?)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn build_where_clause_with_multiple_indices() {
        use crate::domain::task::Index;

        let filter = Filter::new().with_indices([Index::new(1).unwrap(), Index::new(2).unwrap()]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE tpr.row_id IN (?,?)");
        assert_eq!(params.len(), 2);
    }

    // filter.uids + filter.indices

    #[test]
    fn build_where_clause_with_uid_and_index() {
        use crate::domain::task::{Index, UniqueID};

        let uid = UniqueID::new();
        let filter = Filter::new()
            .with_uids([uid])
            .with_indices([Index::new(1).unwrap()]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert_eq!(clause, "WHERE (t.id IN (?) OR tpr.row_id IN (?))");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn build_where_clause_with_uid_and_index_and_status() {
        use crate::domain::task::{Index, UniqueID};

        let uid = UniqueID::new();
        let filter = Filter::new()
            .with_uids([uid])
            .with_indices([Index::new(1).unwrap()])
            .with_statuses([Status::Pending]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert!(clause.starts_with("WHERE "));
        assert!(clause.contains("(t.id IN (?) OR tpr.row_id IN (?))"));
        assert!(clause.contains(" AND "));
        assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
        assert_eq!(params.len(), 2);
    }

    // filter.uids + filter.statuses

    #[test]
    fn build_where_clause_with_uid_and_status() {
        use crate::domain::task::UniqueID;

        let uid = UniqueID::new();
        let filter = Filter::new()
            .with_uids([uid])
            .with_statuses([Status::Pending]);

        let (clause, params) = build_where_clause(&filter).unwrap();
        assert!(clause.starts_with("WHERE "));
        assert!(clause.contains("t.id IN (?)"));
        assert!(clause.contains(" AND "));
        assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
        assert_eq!(params.len(), 1);
    }
}
