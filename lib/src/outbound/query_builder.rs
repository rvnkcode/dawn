use crate::domain::task::{Filter, Status, TaskModification, UniqueID};
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
    if let Some((words_clause, words_params)) = build_words_clause(filter) {
        clauses.push(words_clause);
        params.extend(words_params);
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

// Trigram tokenizer requires ≥ 3 characters
const FTS_MIN_CHARS: usize = 3;

/*
* Escapes a term for FTS5 query by wrapping in double quotes
* and escaping internal double quotes
*/
fn escape_fts5_term(term: &str) -> String {
    let escaped = term.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

/*
* Escapes `%`, `_`, and `\` in a SQLite LIKE pattern.
* Use with `LIKE ? ESCAPE '\'`.
*/
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

pub(crate) fn build_update_clause(
    modification: &TaskModification,
    targets: &[UniqueID],
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
            (
                "completed = ?".to_string(),
                Box::new(c.as_ref().map(|ts| ts.as_seconds())) as Box<dyn ToSql>,
            )
        }),
        modification.deleted.as_ref().map(|d| {
            (
                "deleted = ?".to_string(),
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
                .map(|uid| Box::new(uid.to_string()) as Box<dyn ToSql>),
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
