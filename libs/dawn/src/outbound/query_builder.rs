use crate::domain::Filter;
use rusqlite::ToSql;

pub fn build_where_clause(filter: &Filter) -> anyhow::Result<(String, Vec<Box<dyn ToSql>>)> {
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();
    if filter.is_empty() {
        return Ok((String::from("ORDER BY t.created_at"), params));
    }

    let mut clause = String::from("WHERE ");
    let mut conditions: Vec<String> = Vec::new();

    // ID-related filters (uids, indices, ranges) grouped with OR
    let id_clause = build_id_clause(filter, &mut params);
    if !id_clause.is_empty() {
        // Parentheses needed only when combining with words (AND)
        if filter.words.is_empty() {
            conditions.push(id_clause);
        } else {
            conditions.push(format!("({})", id_clause));
        }
    }

    // Words filter with FTS5 (description search)
    if !filter.words.is_empty() {
        conditions.push("t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)".to_string());
        // FTS5: words joined by space = AND search
        params.push(Box::new(filter.words.join(" ")));
    }

    clause.push_str(&conditions.join(" AND "));
    clause.push_str(" ORDER BY t.created_at");
    Ok((clause, params))
}

fn build_id_clause(filter: &Filter, params: &mut Vec<Box<dyn ToSql>>) -> String {
    let mut id_conditions: Vec<String> = Vec::new();

    if !filter.uids.is_empty() {
        id_conditions.push(format!("t.id IN ({})", repeat_vars(filter.uids.len())));
        for uid in &filter.uids {
            params.push(Box::new(uid.to_string()));
        }
    }

    if !filter.indices.is_empty() {
        id_conditions.push(format!(
            "tpr.row_id IN ({})",
            repeat_vars(filter.indices.len())
        ));
        for idx in &filter.indices {
            params.push(Box::new(idx.get()));
        }
    }

    if !filter.ranges.is_empty() {
        for range in &filter.ranges {
            id_conditions.push("(tpr.row_id BETWEEN ? AND ?)".to_string());
            params.push(Box::new(range.start().get()));
            params.push(Box::new(range.end().get()));
        }
    }

    id_conditions.join(" OR ")
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut vars = "?,".repeat(count);
    vars.pop(); // Remove trailing comma
    vars
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::filter::IndexRange;
    use crate::domain::task::{Index, UniqueID};

    #[test]
    fn empty_filter_returns_order_only() {
        let filter = Filter::default();
        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(clause, "ORDER BY t.created_at");
        assert!(params.is_empty());
    }

    #[test]
    fn single_uid_filter() {
        let mut filter = Filter::default();
        filter.uids.push(UniqueID::from_str("abc12345678").unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(clause, "WHERE t.id IN (?) ORDER BY t.created_at");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn multiple_uids_filter() {
        let mut filter = Filter::default();
        filter.uids.push(UniqueID::from_str("abc12345678").unwrap());
        filter.uids.push(UniqueID::from_str("def12345678").unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(clause, "WHERE t.id IN (?,?) ORDER BY t.created_at");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn single_index_filter() {
        let mut filter = Filter::default();
        filter.indices.push(Index::new(1).unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(clause, "WHERE tpr.row_id IN (?) ORDER BY t.created_at");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn multiple_indices_filter() {
        let mut filter = Filter::default();
        filter.indices.push(Index::new(1).unwrap());
        filter.indices.push(Index::new(2).unwrap());
        filter.indices.push(Index::new(3).unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(clause, "WHERE tpr.row_id IN (?,?,?) ORDER BY t.created_at");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn single_range_filter() {
        let mut filter = Filter::default();
        let range = IndexRange::new(Index::new(1).unwrap(), Index::new(5).unwrap()).unwrap();
        filter.ranges.push(range);

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(
            clause,
            "WHERE (tpr.row_id BETWEEN ? AND ?) ORDER BY t.created_at"
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn multiple_ranges_filter() {
        let mut filter = Filter::default();
        filter
            .ranges
            .push(IndexRange::new(Index::new(1).unwrap(), Index::new(3).unwrap()).unwrap());
        filter
            .ranges
            .push(IndexRange::new(Index::new(5).unwrap(), Index::new(7).unwrap()).unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(
            clause,
            "WHERE (tpr.row_id BETWEEN ? AND ?) OR (tpr.row_id BETWEEN ? AND ?) ORDER BY t.created_at"
        );
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn words_filter() {
        let mut filter = Filter::default();
        filter.words.push("hello".to_string());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(
            clause,
            "WHERE t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?) ORDER BY t.created_at"
        );
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn multiple_words_joined_with_space() {
        let mut filter = Filter::default();
        filter.words.push("hello".to_string());
        filter.words.push("world".to_string());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert!(clause.contains("task_fts MATCH ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn combined_id_filters_use_or() {
        let mut filter = Filter::default();
        filter.uids.push(UniqueID::from_str("abc12345678").unwrap());
        filter.indices.push(Index::new(1).unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(
            clause,
            "WHERE t.id IN (?) OR tpr.row_id IN (?) ORDER BY t.created_at"
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn id_and_words_combined_with_and() {
        let mut filter = Filter::default();
        filter.indices.push(Index::new(1).unwrap());
        filter.words.push("hello".to_string());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert!(clause.contains("(tpr.row_id IN (?))"));
        assert!(clause.contains(" AND "));
        assert!(clause.contains("task_fts MATCH ?"));
        assert_eq!(params.len(), 2);
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
    #[should_panic]
    fn repeat_vars_zero_panics() {
        repeat_vars(0);
    }
}
