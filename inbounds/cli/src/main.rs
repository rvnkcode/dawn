use dawn::outbound::SQLite;

fn main() -> anyhow::Result<()> {
    let _db = SQLite::new()?;
    Ok(())
}
