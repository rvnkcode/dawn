use dawn::outbound::SQLite;

fn main() -> anyhow::Result<()> {
    let db = SQLite::new()?;
    Ok(db.initialize()?)
}
