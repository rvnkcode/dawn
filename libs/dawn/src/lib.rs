pub mod outbound;

use crate::outbound::SQLite;

pub fn bootstrap() -> anyhow::Result<()> {
    let _db = SQLite::new()?;
    Ok(())
}
