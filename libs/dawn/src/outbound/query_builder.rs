use crate::domain::{
    Filter,
    task::{TaskModification, UniqueID},
};
use rusqlite::ToSql;

/// Escapes a term for FTS5 query by wrapping in double quotes
/// and escaping internal double quotes.
fn escape_fts5_term(term: &str) -> String {
    let escaped = term.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

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
        // FTS5: escaped words joined by space = AND search
        let escaped: Vec<String> = filter.words.iter().map(|w| escape_fts5_term(w)).collect();
        params.push(Box::new(escaped.join(" ")));
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

pub fn build_update_clause(
    modification: TaskModification,
    targets: &[&UniqueID],
) -> anyhow::Result<(String, Vec<Box<dyn ToSql>>)> {
    if modification.is_empty() {
        return Err(anyhow::anyhow!("No modifications specified"));
    }
    if targets.is_empty() {
        return Err(anyhow::anyhow!("No target tasks specified"));
    }
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();
    let mut clause = String::from("UPDATE task SET ");
    let mut updates: Vec<String> = Vec::new();

    if let Some(new_description) = modification.description {
        updates.push("description = ?".to_string());
        params.push(Box::new(new_description.to_string()));
    }
    clause.push_str(&updates.join(", "));
    clause.push_str(" WHERE id IN (");
    clause.push_str(&repeat_vars(targets.len()));
    clause.push(')');
    for uid in targets {
        params.push(Box::new(uid.to_string()));
    }
    Ok((clause, params))
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

    #[test]
    fn escape_fts5_term_simple() {
        assert_eq!(escape_fts5_term("hello"), "\"hello\"");
    }

    #[test]
    fn escape_fts5_term_with_quotes() {
        assert_eq!(
            escape_fts5_term("hello \"world\""),
            "\"hello \"\"world\"\"\""
        );
    }

    #[test]
    fn escape_fts5_term_with_special_chars() {
        assert_eq!(escape_fts5_term("test* OR admin"), "\"test* OR admin\"");
    }

    #[test]
    fn escape_fts5_term_empty() {
        assert_eq!(escape_fts5_term(""), "\"\"");
    }

    #[test]
    fn update_clause_empty_modification_returns_error() {
        let modification = TaskModification { description: None };
        let uid = UniqueID::from_str("abc12345678").unwrap();
        let targets = vec![&uid];

        let result = build_update_clause(modification, &targets);

        let err = result.err().expect("Expected error");
        assert_eq!(err.to_string(), "No modifications specified");
    }

    #[test]
    fn update_clause_empty_targets_returns_error() {
        use crate::domain::task::Description;

        let modification = TaskModification {
            description: Some(Description::new("new description").unwrap()),
        };
        let targets: Vec<&UniqueID> = vec![];

        let result = build_update_clause(modification, &targets);

        let err = result.err().expect("Expected error");
        assert_eq!(err.to_string(), "No target tasks specified");
    }

    #[test]
    fn update_clause_single_target_with_description() {
        use crate::domain::task::Description;

        let modification = TaskModification {
            description: Some(Description::new("updated task").unwrap()),
        };
        let uid = UniqueID::from_str("abc12345678").unwrap();
        let targets = vec![&uid];

        let (clause, params) = build_update_clause(modification, &targets).unwrap();

        assert_eq!(clause, "UPDATE task SET description = ? WHERE id IN (?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn update_clause_multiple_targets_with_description() {
        use crate::domain::task::Description;

        let modification = TaskModification {
            description: Some(Description::new("bulk update").unwrap()),
        };
        let uid1 = UniqueID::from_str("abc12345678").unwrap();
        let uid2 = UniqueID::from_str("def12345678").unwrap();
        let uid3 = UniqueID::from_str("ghi12345678").unwrap();
        let targets = vec![&uid1, &uid2, &uid3];

        let (clause, params) = build_update_clause(modification, &targets).unwrap();

        assert_eq!(
            clause,
            "UPDATE task SET description = ? WHERE id IN (?,?,?)"
        );
        assert_eq!(params.len(), 4); // 1 description + 3 target IDs
    }
}
