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

// extract UID from `info` output
pub fn extract_uid(stdout: &str) -> String {
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("UID")
            && let Some(token) = rest.split_whitespace().next()
            && token.len() == 12
            && token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return token.to_string();
        }
    }
    panic!("UID row not found in info output:\n{stdout}");
}

pub fn run_stdout(cmd: &mut Command) -> String {
    let out = cmd.output().expect("run");
    assert!(
        out.status.success(),
        "command failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

pub fn assert_empty_filter_aborts(args: &[&str]) {
    let (_dir, db) = test_db();
    let out = dawn_cmd(&db).args(args).output().expect("run");
    assert_eq!(out.status.code(), Some(2), "expected exit 2");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("Command prevented from running."),
        "stderr missing abort message: {stderr}"
    );
}
