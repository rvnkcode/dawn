use rusqlite::Connection;
use std::{fs, path::PathBuf};

const DB_VERSION: u8 = 1;

// TODO: Remove dead_code allowance
#[allow(dead_code)]
pub struct SQLite {
    conn: Connection,
}

impl SQLite {
    pub fn new() -> anyhow::Result<Self> {
        let conn = Self::connect()?;
        Self::initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    fn get_path() -> anyhow::Result<PathBuf> {
        match dirs::home_dir() {
            Some(home_dir) => {
                let path = home_dir.join(".dawn");
                fs::create_dir_all(&path)?;
                Ok(path.join("dawn.db"))
            }
            None => Err(anyhow::anyhow!("Could not determine home directory")),
        }
    }

    fn connect() -> anyhow::Result<Connection> {
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
