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

// Tasks share `entry` seconds, so the Index↔description mapping is not stable
// (tiebreaker is the random UID lex order). Tests must not assume that the
// i-th description receives Index i; filter on all seeded indices, or assert
// on counts rather than on which description maps to which index.
pub fn setup_tasks(db: &Path, descriptions: &[&str]) {
    for desc in descriptions {
        dawn_cmd(db).args(["add", desc]).assert().success();
    }
}
