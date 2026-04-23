#![allow(dead_code)]

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub fn dawn_cmd(db: &Path) -> Command {
    let mut cmd = Command::cargo_bin("dawn").expect("binary 'dawn' from dawn-cli crate");
    cmd.env("DAWN_DB_PATH", db).env_remove("XDG_DATA_HOME");
    cmd
}

pub fn test_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    (dir, db)
}

pub fn setup_tasks(db: &Path, descriptions: &[&str]) {
    for (i, desc) in descriptions.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        dawn_cmd(db).args(["add", desc]).assert().success();
    }
}
