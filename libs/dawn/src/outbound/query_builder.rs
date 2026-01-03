use crate::domain::{
    Filter,
    task::{TaskModification, UniqueID},
};
use rusqlite::ToSql;

pub fn build_where_clause(filter: &Filter) -> anyhow::Result<(String, Vec<Box<dyn ToSql>>)> {
    if filter.is_empty() {
        return Ok((String::from("ORDER BY t.created_at"), Vec::new()));
    }

    let (id_clause, id_params) = build_id_clause(filter);
    let (words_clause, words_params) = build_words_clause(filter);

    // ID clause needs parentheses only when combining with words (AND)
    let conditions: Vec<String> = [
        id_clause.map(|id| match words_clause {
            Some(_) => format!("({})", id),
            None => id,
        }),
        words_clause,
    ]
    .into_iter()
    .flatten()
    .collect();

    let params: Vec<Box<dyn ToSql>> = id_params.into_iter().chain(words_params).collect();

    Ok((
        format!("WHERE {} ORDER BY t.created_at", conditions.join(" AND ")),
        params,
    ))
}

fn build_id_clause(filter: &Filter) -> (Option<String>, Vec<Box<dyn ToSql>>) {
    let uid_clause: Option<(String, Vec<Box<dyn ToSql>>)> = (!filter.uids.is_empty()).then(|| {
        let params: Vec<Box<dyn ToSql>> = filter
            .uids
            .iter()
            .map(|uid| Box::new(uid.to_string()) as Box<dyn ToSql>)
            .collect();
        (
            format!("t.id IN ({})", repeat_vars(filter.uids.len())),
            params,
        )
    });

    let index_clause: Option<(String, Vec<Box<dyn ToSql>>)> =
        (!filter.indices.is_empty()).then(|| {
            let params: Vec<Box<dyn ToSql>> = filter
                .indices
                .iter()
                .map(|idx| Box::new(idx.get()) as Box<dyn ToSql>)
                .collect();
            (
                format!("tpr.row_id IN ({})", repeat_vars(filter.indices.len())),
                params,
            )
        });

    let range_clauses: Vec<(String, Vec<Box<dyn ToSql>>)> = filter
        .ranges
        .iter()
        .map(|range| {
            (
                "(tpr.row_id BETWEEN ? AND ?)".to_string(),
                vec![
                    Box::new(range.start().get()) as Box<dyn ToSql>,
                    Box::new(range.end().get()) as Box<dyn ToSql>,
                ],
            )
        })
        .collect();

    let all_clauses: Vec<(String, Vec<Box<dyn ToSql>>)> = uid_clause
        .into_iter()
        .chain(index_clause)
        .chain(range_clauses)
        .collect();
    if all_clauses.is_empty() {
        return (None, Vec::new());
    }

    let (clauses, params): (Vec<String>, Vec<Vec<Box<dyn ToSql>>>) =
        all_clauses.into_iter().unzip();

    (
        Some(clauses.join(" OR ")),
        params.into_iter().flatten().collect(),
    )
}

fn build_words_clause(filter: &Filter) -> (Option<String>, Vec<Box<dyn ToSql>>) {
    if filter.words.is_empty() {
        return (None, Vec::new());
    }

    let escaped = filter
        .words
        .iter()
        .map(|w| escape_fts5_term(w))
        .collect::<Vec<_>>()
        .join(" ");

    (
        Some("t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)".to_string()),
        vec![Box::new(escaped) as Box<dyn ToSql>],
    )
}

/// Escapes a term for FTS5 query by wrapping in double quotes
/// and escaping internal double quotes.
fn escape_fts5_term(term: &str) -> String {
    let escaped = term.replace('"', "\"\"");
    format!("\"{}\"", escaped)
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

    // Build updates from modification fields
    let (updates, update_params): (Vec<&str>, Vec<Box<dyn ToSql>>) =
        [modification.description.map(|desc| {
            (
                "description = ?",
                Box::new(desc.to_string()) as Box<dyn ToSql>,
            )
        })]
        .into_iter()
        .flatten()
        .unzip();

    let target_params: Vec<Box<dyn ToSql>> = targets
        .iter()
        .map(|uid| Box::new(uid.to_string()) as Box<dyn ToSql>)
        .collect();

    let params: Vec<Box<dyn ToSql>> = update_params.into_iter().chain(target_params).collect();

    let clause = format!(
        "UPDATE task SET {} WHERE id IN ({})",
        updates.join(", "),
        repeat_vars(targets.len())
    );

    Ok((clause, params))
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    std::iter::repeat_n("?", count)
        .collect::<Vec<_>>()
        .join(",")
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
        filter.uids.push("abc12345678".parse::<UniqueID>().unwrap());

        let (clause, params) = build_where_clause(&filter).unwrap();

        assert_eq!(clause, "WHERE t.id IN (?) ORDER BY t.created_at");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn multiple_uids_filter() {
        let mut filter = Filter::default();
        filter.uids.push("abc12345678".parse::<UniqueID>().unwrap());
        filter.uids.push("def12345678".parse::<UniqueID>().unwrap());

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
        filter.uids.push("abc12345678".parse::<UniqueID>().unwrap());
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
        let modification = TaskModification {
            description: None,
            completed_at: None,
        };
        let uid = "abc12345678".parse::<UniqueID>().unwrap();
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
            completed_at: None,
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
            completed_at: None,
        };
        let uid = "abc12345678".parse::<UniqueID>().unwrap();
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
            completed_at: None,
        };
        let uid1 = "abc12345678".parse::<UniqueID>().unwrap();
        let uid2 = "def12345678".parse::<UniqueID>().unwrap();
        let uid3 = "ghi12345678".parse::<UniqueID>().unwrap();
        let targets = vec![&uid1, &uid2, &uid3];

        let (clause, params) = build_update_clause(modification, &targets).unwrap();

        assert_eq!(
            clause,
            "UPDATE task SET description = ? WHERE id IN (?,?,?)"
        );
        assert_eq!(params.len(), 4); // 1 description + 3 target IDs
    }
}
