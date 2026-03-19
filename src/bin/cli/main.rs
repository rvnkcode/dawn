use dawn::outbound::SQLite;

fn main() -> anyhow::Result<()> {
    let mut db = SQLite::new()?;
    db.initialize()?;
    Ok(())
}
