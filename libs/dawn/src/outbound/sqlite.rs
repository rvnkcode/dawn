use rusqlite::Connection;
use std::{fs, path::PathBuf};

use crate::domain::task::port::TaskRepository;

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
        if user_version < DB_VERSION {
            conn.execute_batch(include_str!("../../sql/schema.sql"))?;
        }
        Ok(())
    }
}

impl TaskRepository for SQLite {}
