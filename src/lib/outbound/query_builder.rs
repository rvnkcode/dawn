use crate::domain::task::{Filter, Status};

const ALL_STATUSES: usize = 3;

pub(crate) fn build_where_clause(filter: &Filter) -> Option<String> {
    if filter.is_empty() {
        return None;
    }
    build_status_clause(filter).map(|clause| format!("WHERE {clause}"))
}

fn build_status_clause(filter: &Filter) -> Option<String> {
    if filter.statuses.is_empty() || filter.statuses.len() == ALL_STATUSES {
        return None;
    }
    let mut conditions = Vec::new();
    for status in &filter.statuses {
        match status {
            Status::Pending => conditions.push("(t.deleted IS NULL AND t.completed IS NULL)"),
            Status::Completed => conditions.push("(t.deleted IS NULL AND t.completed IS NOT NULL)"),
            Status::Deleted => conditions.push("(t.deleted IS NOT NULL)"),
        }
    }
    Some(conditions.join(" OR "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn build_where_clause_with_empty_filter() {
        let filter = Filter {
            statuses: HashSet::new(),
        };

        assert_eq!(build_where_clause(&filter), None);
    }

    #[test]
    fn build_where_clause_with_pending_only() {
        let filter = Filter {
            statuses: HashSet::from([Status::Pending]),
        };

        assert_eq!(
            build_where_clause(&filter),
            Some("WHERE (t.deleted IS NULL AND t.completed IS NULL)".to_string())
        );
    }

    #[test]
    fn build_where_clause_with_completed_only() {
        let filter = Filter {
            statuses: HashSet::from([Status::Completed]),
        };

        assert_eq!(
            build_where_clause(&filter),
            Some("WHERE (t.deleted IS NULL AND t.completed IS NOT NULL)".to_string())
        );
    }

    #[test]
    fn build_where_clause_with_deleted_only() {
        let filter = Filter {
            statuses: HashSet::from([Status::Deleted]),
        };

        assert_eq!(
            build_where_clause(&filter),
            Some("WHERE (t.deleted IS NOT NULL)".to_string())
        );
    }

    #[test]
    fn build_where_clause_with_two_statuses() {
        let filter = Filter {
            statuses: HashSet::from([Status::Pending, Status::Completed]),
        };

        let clause = build_where_clause(&filter).unwrap();
        assert!(clause.starts_with("WHERE "));
        assert!(clause.contains(" OR "));
        assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
        assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NOT NULL)"));
    }

    #[test]
    fn build_where_clause_with_all_statuses() {
        let filter = Filter {
            statuses: HashSet::from([Status::Pending, Status::Completed, Status::Deleted]),
        };

        assert_eq!(build_where_clause(&filter), None);
    }
}
