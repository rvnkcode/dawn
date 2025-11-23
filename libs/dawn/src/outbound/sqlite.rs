use crate::domain::task::{Description, Index, Task, UniqueID, port::TaskRepository};
use rusqlite::{Connection, params};
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
        let path = dirs::home_dir().unwrap_or(PathBuf::from(".")).join(".dawn");
        fs::create_dir_all(&path)?;
        Ok(path.join("dawn.db"))
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
    fn create_task(&self, id: UniqueID, description: Description) -> anyhow::Result<Task> {
        self.conn.execute(
            "INSERT INTO task (id, description) VALUES (?1, ?2)",
            params![id.to_string(), description.to_string()],
        )?;
        let count = self.count_pending_tasks();
        Ok(Task {
            uid: id,
            index: Index::new(count)?,
            description,
        })
    }

    fn count_pending_tasks(&self) -> usize {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM task_pending_row_id", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        count
    }
}
