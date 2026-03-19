use dawn::outbound::SQLite;

fn main() -> anyhow::Result<()> {
    let mut db = SQLite::new()?;
    Ok(db.initialize()?)
}
