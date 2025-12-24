use crate::domain::Filter;
use rusqlite::ToSql;

pub struct QueryBuilder;

impl QueryBuilder {
    pub fn build_where_clause(filter: &Filter) -> anyhow::Result<(String, Vec<Box<dyn ToSql>>)> {
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();
        if filter.is_empty() {
            return Ok((String::from("ORDER BY t.created_at"), params));
        }

        let mut clause = String::from("WHERE ");
        let mut conditions: Vec<String> = Vec::new();

        // ID-related filters (uids, indices, ranges) grouped with OR
        let id_clause = Self::build_id_clause(filter, &mut params);
        if !id_clause.is_empty() {
            conditions.push(format!("({})", id_clause));
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
            id_conditions.push(format!(
                "t.id IN ({})",
                Self::repeat_vars(filter.uids.len())
            ));
            for uid in &filter.uids {
                params.push(Box::new(uid.to_string()));
            }
        }

        if !filter.indices.is_empty() {
            id_conditions.push(format!(
                "tpr.row_id IN ({})",
                Self::repeat_vars(filter.indices.len())
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
}
