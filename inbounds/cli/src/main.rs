fn main() -> anyhow::Result<()> {
    dawn::bootstrap()?;
    println!("SQLite database initialized successfully.");
    Ok(())
}
