use assert_cmd::Command;

pub fn dawn_cmd(db: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("dawn").expect("binary built with --features cli");
    cmd.env("DAWN_DB_PATH", db).env_remove("XDG_DATA_HOME");
    cmd
}
