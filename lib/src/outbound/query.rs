use anyhow::Context;
use rusqlite::ToSql;
use uuid::Uuid;

use crate::domain::task::{Direction, Filter, SortKey, Status, TaskModification, UuidPrefix};

type Clause = (String, Vec<Box<dyn ToSql>>);

pub(crate) fn build_where_clause(filter: &Filter) -> anyhow::Result<Option<Clause>> {
    if filter.is_empty() {
        return Ok(None);
    }
    let mut clauses = Vec::new();
    let mut params = Vec::new();
    // UUID, index, and index range combined with OR
    if let Some((id_clause, id_params)) = build_id_clause(filter)? {
        clauses.push(id_clause);
        params.extend(id_params);
    }
    if let Some(status_clause) = build_status_clause(filter) {
        clauses.push(status_clause);
    }
    // Words combined with AND
    if let Some((words_clause, words_params)) = build_words_clause(filter) {
        clauses.push(words_clause);
        params.extend(words_params);
    }

    Ok(Some((format!("WHERE {}", clauses.join(" AND ")), params)))
}

fn build_id_clause(filter: &Filter) -> anyhow::Result<Option<Clause>> {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some((uuid_clause, uuid_params)) = build_uuid_clause(filter) {
        clauses.push(uuid_clause);
        params.extend(uuid_params);
    }
    if let Some((indices_clause, indices_params)) = build_indices_clause(filter)? {
        clauses.push(indices_clause);
        params.extend(indices_params);
    }
    for (range_clause, range_params) in build_index_range_clauses(filter)? {
        clauses.push(range_clause);
        params.extend(range_params);
    }

    Ok(match clauses.len() {
        0 => None,
        1 => Some((clauses.remove(0), params)),
        _ => Some((format!("({})", clauses.join(" OR ")), params)),
    })
}

fn build_uuid_clause(filter: &Filter) -> Option<Clause> {
    let uuids = filter.uuids();
    if uuids.is_empty() {
        return None;
    }

    let (full, prefix): (Vec<&UuidPrefix>, Vec<&UuidPrefix>) =
        uuids.iter().partition(|u| u.is_full());

    let mut sub_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    // Full UUID no need for LIKE: exact match on t.id
    if !full.is_empty() {
        sub_clauses.push(format!("t.id IN ({})", repeat_vars(full.len())));
        params.extend(
            full.iter()
                .map(|u| Box::new(u.to_string()) as Box<dyn ToSql>),
        );
    }
    for uuid in &prefix {
        sub_clauses.push("t.id LIKE ?".to_string());
        // No LIKE escape needed: UuidPrefix [0-9a-f-] excludes %, _, \
        params.push(Box::new(format!("{uuid}%")) as Box<dyn ToSql>);
    }

    let clause = if sub_clauses.len() == 1 {
        sub_clauses.remove(0)
    } else {
        format!("({})", sub_clauses.join(" OR "))
    };
    Some((clause, params))
}

fn build_indices_clause(filter: &Filter) -> anyhow::Result<Option<Clause>> {
    let indices = filter.indices();
    if indices.is_empty() {
        return Ok(None);
    }

    let clause = format!("tpr.row_id IN ({})", repeat_vars(indices.len()));
    let params: Vec<Box<dyn ToSql>> = indices
        .iter()
        .map(|index| {
            let v = i64::try_from(index.get()).context("task index exceeds i64 range")?;
            Ok(Box::new(v) as Box<dyn ToSql>)
        })
        .collect::<anyhow::Result<_>>()?;
    Ok(Some((clause, params)))
}

fn build_index_range_clauses(filter: &Filter) -> anyhow::Result<Vec<Clause>> {
    filter
        .index_ranges()
        .iter()
        .map(|range| {
            let start =
                i64::try_from(range.start().get()).context("task index exceeds i64 range")?;
            let end = i64::try_from(range.end().get()).context("task index exceeds i64 range")?;
            let params: Vec<Box<dyn ToSql>> = vec![
                Box::new(start) as Box<dyn ToSql>,
                Box::new(end) as Box<dyn ToSql>,
            ];
            Ok(("tpr.row_id BETWEEN ? AND ?".to_string(), params))
        })
        .collect()
}

fn build_status_clause(filter: &Filter) -> Option<String> {
    let clause = match filter.report_status()? {
        Status::Pending => "(t.deleted IS NULL AND t.completed IS NULL)",
        Status::Completed => "(t.deleted IS NULL AND t.completed IS NOT NULL)",
        Status::Deleted => "t.deleted IS NOT NULL",
    };
    Some(clause.to_string())
}

// Trigram tokenizer requires ≥ 3 characters
const FTS_MIN_CHARS: usize = 3;

fn build_words_clause(filter: &Filter) -> Option<Clause> {
    let words = filter.words();
    if words.is_empty() {
        return None;
    }

    let (long, short): (Vec<&String>, Vec<&String>) = words
        .iter()
        .partition(|w| w.chars().count() >= FTS_MIN_CHARS);

    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    // FTS5 MATCH
    if !long.is_empty() {
        let match_query = long
            .iter()
            .map(|w| escape_fts5_term(w))
            .collect::<Vec<_>>()
            .join(" AND ");
        clauses.push("t.id IN (SELECT id FROM task_fts WHERE task_fts MATCH ?)".to_string());
        params.push(Box::new(match_query) as Box<dyn ToSql>);
    }

    // LIKE (under 3 chars)
    for w in &short {
        clauses.push(r"t.description LIKE ? ESCAPE '\'".to_string());
        params.push(Box::new(format!("%{}%", escape_like(w))) as Box<dyn ToSql>);
    }

    let joined = if clauses.len() == 1 {
        // move single clause out of clauses vector to avoid unnecessary parentheses
        clauses.remove(0)
    } else {
        format!("({})", clauses.join(" AND "))
    };
    Some((joined, params))
}

// Escapes a term for FTS5 query by wrapping in double quotes
// and escaping internal double quotes
fn escape_fts5_term(term: &str) -> String {
    let escaped = term.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

// Escapes `%`, `_`, and `\` in a SQLite LIKE pattern.
// Use with `LIKE ? ESCAPE '\'`.
fn escape_like(term: &str) -> String {
    let mut out = String::new();
    for c in term.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

pub(crate) fn build_order_clause(filter: &Filter) -> String {
    const DEFAULT_ORDER: &str = "ORDER BY t.entry ASC, t.id ASC";
    const ID_TIEBREAKER: &str = "t.id ASC";

    match filter.sort_key() {
        // Always tiebreak on t.id ASC for deterministic ordering when the key ties.
        Some(key) => format!("ORDER BY {}, {ID_TIEBREAKER}", serialize_sort_key(key)),
        None => DEFAULT_ORDER.to_string(),
    }
}

fn serialize_sort_key(key: &SortKey) -> String {
    let (column, direction) = match key {
        SortKey::Entry(d) => ("t.entry", d),
        SortKey::Completed(d) => ("t.completed", d),
    };
    let direction = match direction {
        Direction::Asc => "ASC",
        Direction::Desc => "DESC",
    };
    format!("{column} {direction}")
}

pub(crate) fn build_update_clause(
    modification: &TaskModification,
    targets: &[Uuid],
) -> anyhow::Result<Clause> {
    if modification.is_empty() {
        return Err(anyhow::anyhow!("no modification specified"));
    }
    if targets.is_empty() {
        return Err(anyhow::anyhow!("no target specified"));
    }

    // Collect update expressions and their parameters based on the provided modification
    let (updates, update_params): (Vec<String>, Vec<Box<dyn ToSql>>) = [
        modification.description.as_ref().map(|desc| {
            (
                "description = ?".to_string(),
                Box::new(desc.to_string()) as Box<dyn ToSql>,
            )
        }),
        modification.completed.as_ref().map(|c| {
            // Use IFNULL to prevent overwriting existing completed timestamp
            let sql = if c.is_some() {
                "completed = IFNULL(completed, ?)"
            } else {
                "completed = ?"
            };
            (
                sql.to_string(),
                Box::new(c.as_ref().map(|ts| ts.as_seconds())) as Box<dyn ToSql>,
            )
        }),
        modification.deleted.as_ref().map(|d| {
            // Use IFNULL to prevent overwriting existing deleted timestamp
            let sql = if d.is_some() {
                "deleted = IFNULL(deleted, ?)"
            } else {
                "deleted = ?"
            };
            (
                sql.to_string(),
                Box::new(d.as_ref().map(|ts| ts.as_seconds())) as Box<dyn ToSql>,
            )
        }),
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
                .map(|uuid| Box::new(uuid.to_string()) as Box<dyn ToSql>),
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

// ref: https://docs.rs/rusqlite/latest/rusqlite/struct.ParamsFromIter.html#realistic-use-case
pub(crate) fn repeat_vars(count: usize) -> String {
    vec!["?"; count].join(",")
}

#[cfg(test)]
mod tests;
