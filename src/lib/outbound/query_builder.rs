use crate::domain::task::{Filter, Status};

const ALL_STATUSES: usize = 3;

pub fn build_where_clause(filter: &Filter) -> String {
    if filter.is_empty() {
        return String::from("ORDER BY t.entry, t.id");
    }
    match build_status_clause(filter) {
        Some(status_clause) => format!("WHERE {} ORDER BY t.entry, t.id", status_clause),
        None => String::from("ORDER BY t.entry, t.id"),
    }
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

        let clause = build_where_clause(&filter);

        assert_eq!(clause, "ORDER BY t.entry, t.id");
    }

    #[test]
    fn build_where_clause_with_pending_only() {
        let filter = Filter {
            statuses: HashSet::from([Status::Pending]),
        };

        let clause = build_where_clause(&filter);

        assert_eq!(
            clause,
            "WHERE (t.deleted IS NULL AND t.completed IS NULL) ORDER BY t.entry, t.id"
        );
    }

    #[test]
    fn build_where_clause_with_completed_only() {
        let filter = Filter {
            statuses: HashSet::from([Status::Completed]),
        };

        let clause = build_where_clause(&filter);

        assert_eq!(
            clause,
            "WHERE (t.deleted IS NULL AND t.completed IS NOT NULL) ORDER BY t.entry, t.id"
        );
    }

    #[test]
    fn build_where_clause_with_deleted_only() {
        let filter = Filter {
            statuses: HashSet::from([Status::Deleted]),
        };

        let clause = build_where_clause(&filter);

        assert_eq!(
            clause,
            "WHERE (t.deleted IS NOT NULL) ORDER BY t.entry, t.id"
        );
    }

    #[test]
    fn build_where_clause_with_two_statuses() {
        let filter = Filter {
            statuses: HashSet::from([Status::Pending, Status::Completed]),
        };

        let clause = build_where_clause(&filter);

        assert!(clause.starts_with("WHERE "));
        assert!(clause.ends_with(" ORDER BY t.entry, t.id"));
        assert!(clause.contains(" OR "));
        assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NULL)"));
        assert!(clause.contains("(t.deleted IS NULL AND t.completed IS NOT NULL)"));
    }

    #[test]
    fn build_where_clause_with_all_statuses() {
        let filter = Filter {
            statuses: HashSet::from([Status::Pending, Status::Completed, Status::Deleted]),
        };

        let clause = build_where_clause(&filter);

        assert_eq!(clause, "ORDER BY t.entry, t.id");
    }
}
