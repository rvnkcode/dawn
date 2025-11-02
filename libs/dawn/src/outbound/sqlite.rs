use anyhow::anyhow;
use rusqlite::Connection;
use std::{fs, path::PathBuf};

pub struct SQLite {
    conn: Connection,
}

impl SQLite {
    pub fn new() -> anyhow::Result<Self> {
        let conn = Self::open_connection()?;
        Ok(SQLite { conn })
    }

    fn get_path() -> anyhow::Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("failed to get home directory"))?;
        let path = home_dir.join(".dawn");
        fs::create_dir_all(&path)?;
        Ok(path.join("dawn.db"))
    }

    fn open_connection() -> anyhow::Result<Connection> {
        let path = Self::get_path()?;
        Ok(Connection::open(path)?)
    }

    // TODO: define and initialize schema
}
