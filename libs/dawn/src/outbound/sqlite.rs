use crate::{
    domain::{
        Filter,
        task::{Description, Index, Task, TaskCreation, UniqueID, port::TaskRepository},
    },
    outbound::QueryBuilder,
};
use rusqlite::{Connection, params, params_from_iter};
use std::{fs, path::PathBuf};

const DB_VERSION: u8 = 1;

pub struct SQLite {
    conn: Connection,
}

impl SQLite {
    pub fn new() -> anyhow::Result<Self> {
        let conn = Self::open_connection()?;
        Self::initialize_schema(&conn)?;
        Ok(SQLite { conn })
    }

    fn get_path() -> anyhow::Result<PathBuf> {
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir.join(".dawn");
            fs::create_dir_all(&path)?;
            Ok(path.join("dawn.db"))
        } else {
            Err(anyhow::anyhow!("Could not determine home directory"))
        }
    }

    fn open_connection() -> anyhow::Result<Connection> {
        let path = Self::get_path()?;
        Ok(Connection::open(path)?)
    }

    fn get_user_version(conn: &Connection) -> u8 {
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap_or(0)
    }

    fn initialize_schema(conn: &Connection) -> anyhow::Result<()> {
        let user_version = Self::get_user_version(conn);
        if user_version != DB_VERSION {
            // TODO: Backup data
            conn.execute_batch(include_str!("../../sql/schema.sql"))?;
            // TODO: Restore data
        }
        Ok(())
    }
}

impl TaskRepository for SQLite {
    // TODO: Other properties e.g. project, tags, etc.
    fn create_task(&self, id: UniqueID, req: TaskCreation) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO task (id, description) VALUES (?1, ?2)",
            params![id.to_string(), req.description.to_string()],
        )?;
        Ok(())
    }

    fn count_pending_tasks(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM task_pending_row_id", [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }

    fn get_pending_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        let select_clause = "SELECT t.id, tpr.row_id, t.description, t.created_at \
            FROM task AS t \
                INNER JOIN task_pending_row_id AS tpr ON tpr.id = t.id";
        let (where_clause, params) = QueryBuilder::build_where_clause(filter)?;
        let query = format!("{} {}", select_clause, where_clause);
        let mut stmt = self.conn.prepare(&query)?;
        let tasks = stmt
            .query_map(params_from_iter(&params), |row| {
                let id_str: String = row.get(0)?;
                let row_id: usize = row.get(1)?;
                let description_str: String = row.get(2)?;
                let created_at: i64 = row.get(3)?;
                Ok((id_str, row_id, description_str, created_at))
            })?
            .map(|result| {
                let (id_str, row_id, description_str, created_at) = result?;
                Ok(Task {
                    uid: UniqueID::from_str(&id_str)?,
                    index: Some(Index::new(row_id)?),
                    description: Description::new(&description_str)?,
                    created_at,
                    completed_at: None,
                    deleted_at: None,
                })
            })
            .collect::<anyhow::Result<Vec<Task>>>()?;
        Ok(tasks)
    }

    // TODO: No index covers `created_at` for all tasks (only partial index for pending).
    // Consider adding `CREATE INDEX idx_task_created_at ON task (created_at)` if performance degrades.
    fn get_all_tasks(&self, filter: &Filter) -> anyhow::Result<Vec<Task>> {
        let select_clause = "SELECT t.id, tpr.row_id, t.description, t.created_at, t.completed_at, t.deleted_at \
            FROM task AS t \
                LEFT JOIN task_pending_row_id AS tpr ON tpr.id = t.id";
        let (where_clause, params) = QueryBuilder::build_where_clause(filter)?;
        let query = format!("{} {}", select_clause, where_clause);
        let mut stmt = self.conn.prepare(&query)?;
        let tasks = stmt
            .query_map(params_from_iter(&params), |row| {
                let id_str: String = row.get(0)?;
                let row_id: Option<usize> = row.get(1)?;
                let description_str: String = row.get(2)?;
                let created_at: i64 = row.get(3)?;
                let completed_at: Option<i64> = row.get(4)?;
                let deleted_at: Option<i64> = row.get(5)?;
                Ok((
                    id_str,
                    row_id,
                    description_str,
                    created_at,
                    completed_at,
                    deleted_at,
                ))
            })?
            .map(|result| {
                let (id_str, row_id, description_str, created_at, completed_at, deleted_at) =
                    result?;
                Ok(Task {
                    uid: UniqueID::from_str(&id_str)?,
                    index: row_id.map(Index::new).transpose()?,
                    description: Description::new(&description_str)?,
                    created_at,
                    completed_at,
                    deleted_at,
                })
            })
            .collect::<anyhow::Result<Vec<Task>>>()?;
        Ok(tasks)
    }
}
