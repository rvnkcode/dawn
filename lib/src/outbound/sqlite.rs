use std::cmp::Ordering;
#[cfg(not(coverage))]
use std::path::{Path, PathBuf};

use rusqlite::{Connection, params_from_iter};
use uuid::Uuid;

use crate::{
    domain::task::{
        Description, Filter, Index, Task, TaskCreation, TaskModification, Timestamp,
        port::TaskRepository,
    },
    outbound::query_builder,
};

const DB_VERSION: u8 = 1;

pub struct SQLite {
    conn: Connection,
}

#[derive(Debug, thiserror::Error)]
pub enum SQLiteError {
    #[error("database initialization error: {0}")]
    InitializationError(String),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

impl SQLite {
    // Excluded from coverage because it depends on the filesystem
    #[cfg(not(coverage))]
    pub fn new() -> Result<Self, SQLiteError> {
        let db_path = get_db_path()?;
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub(crate) fn new_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    // mut self to use transaction
    pub fn initialize(&mut self) -> Result<(), SQLiteError> {
        let user_version = self.get_user_version()?;
        match user_version.cmp(&DB_VERSION) {
            Ordering::Less => {
                // TODO: Backup data
                let tx = self.conn.transaction()?;
                tx.execute_batch(include_str!("./schema.sql"))?;
                tx.commit()?;
                // TODO: Restore data
            }
            Ordering::Equal => {} // Do nothing
            Ordering::Greater => {
                return Err(SQLiteError::InitializationError(format!(
                    "database version ({user_version}) is newer than supported version ({DB_VERSION})"
                )));
            }
        }
        Ok(())
    }

    fn get_user_version(&self) -> Result<u8, SQLiteError> {
        let user_version = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(user_version)
    }
}

// Excluded from coverage because it depends on the filesystem
#[cfg(not(coverage))]
fn get_db_path() -> Result<PathBuf, SQLiteError> {
    // Uses env var for E2E tests to isolate test data from real data
    if let Ok(override_path) = std::env::var("DAWN_DB_PATH") {
        let path = PathBuf::from(override_path);
        ensure_parent_dir(&path)?;
        return Ok(path);
    }
    /*
     * linux: ~/.local
     * macOS: ~/Library/Application\ Support
     */
    let data_dir = dirs::data_local_dir().ok_or(SQLiteError::InitializationError(
        "could not determine local data directory".into(),
    ))?;
    let dawn_dir = data_dir.join("dawn");
    let path = dawn_dir.join("dawn.db");
    ensure_parent_dir(&path)?;
    Ok(path)
}

// Excluded from coverage because it depends on the filesystem
#[cfg(not(coverage))]
fn ensure_parent_dir(path: &Path) -> Result<(), SQLiteError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            SQLiteError::InitializationError(format!(
                "failed to create directory '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }
    Ok(())
}

impl TaskRepository for SQLite {
    fn create_task(&self, id: &Uuid, req: &TaskCreation) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO task (id, description) VALUES (?, ?)",
            [id.to_string(), req.description.to_string()],
        )?;
        Ok(())
    }

    fn count_pending(&self) -> anyhow::Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM task WHERE deleted IS NULL AND completed IS NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(count.try_into()?)
    }

    fn list_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        let select_clause = "SELECT t.id, tpr.row_id, t.description, t.entry, t.completed, t.deleted, t.modified \
            FROM task AS t \
                LEFT JOIN vw_task_pending_row_id AS tpr ON tpr.id = t.id";
        let order_clause = "ORDER BY t.entry, t.id";
        let (query, params) = match query_builder::build_where_clause(filter)? {
            Some((where_clause, params)) => (
                format!("{select_clause} {where_clause} {order_clause}"),
                params,
            ),
            None => (format!("{select_clause} {order_clause}"), Vec::new()),
        };
        let mut stmt = self.conn.prepare(&query)?;
        let tasks = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                let id_str: String = row.get(0)?;
                let row_id: Option<i64> = row.get(1)?;
                let description_str: String = row.get(2)?;
                let entry: i64 = row.get(3)?;
                let completed: Option<i64> = row.get(4)?;
                let deleted: Option<i64> = row.get(5)?;
                let modified: i64 = row.get(6)?;
                Ok((
                    id_str,
                    row_id,
                    description_str,
                    entry,
                    completed,
                    deleted,
                    modified,
                ))
            })?
            .map(|result| {
                let (id_str, row_id, description_str, entry, completed, deleted, modified) =
                    result?;
                Ok(Task {
                    uuid: Uuid::parse_str(&id_str)?,
                    index: match row_id {
                        Some(id) => Some(Index::new(id.try_into()?)?),
                        None => None,
                    },
                    description: Description::new(&description_str)?,
                    entry: Timestamp::new(entry)?,
                    completed: completed.map(Timestamp::new).transpose()?,
                    deleted: deleted.map(Timestamp::new).transpose()?,
                    modified: Timestamp::new(modified)?,
                })
            })
            .collect::<anyhow::Result<Vec<Task>>>()?;
        Ok(tasks)
    }

    fn update_tasks(
        &self,
        modification: &TaskModification,
        targets: &[Uuid],
    ) -> anyhow::Result<()> {
        let (query, params) = query_builder::build_update_clause(modification, targets)?;
        self.conn.execute(&query, params_from_iter(params.iter()))?;
        Ok(())
    }

    fn delete_tasks(&self, targets: &[Uuid]) -> anyhow::Result<()> {
        if targets.is_empty() {
            return Err(anyhow::anyhow!("no target IDs provided for deletion"));
        }
        let query = format!(
            "DELETE FROM task WHERE id IN ({})",
            query_builder::repeat_vars(targets.len())
        );
        let params = targets
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>();
        self.conn.execute(&query, params_from_iter(params.iter()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
