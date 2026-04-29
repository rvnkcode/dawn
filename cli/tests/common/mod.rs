#![allow(dead_code)]

use assert_cmd::Command;
use rexpect::process::WaitStatus;
use rexpect::session::{PtySession, spawn_command};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const PTY_TIMEOUT_MS: u64 = 5000;
const SELECT_DOWN: &str = "\x1b[B";

pub fn dawn_cmd(db: &Path) -> Command {
    let mut cmd = Command::cargo_bin("dawn").expect("binary 'dawn' from dawn-cli crate");
    cmd.env("DAWN_DB_PATH", db).env_remove("XDG_DATA_HOME");
    cmd
}

pub fn dawn_pty(db: &Path, args: &[&str]) -> PtySession {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_dawn"));
    cmd.env("DAWN_DB_PATH", db).env_remove("XDG_DATA_HOME");
    cmd.args(args);
    spawn_command(cmd, Some(PTY_TIMEOUT_MS)).expect("spawn dawn under PTY")
}

// Drives a single-task delete flow under PTY (Yes on the Confirm prompt) and
// waits for the success footer. Used by tests that need a deleted task as
// fixture rather than as the system under test.
pub fn delete_via_pty(db: &Path, target: &str) {
    let mut p = dawn_pty(db, &[target, "delete"]);
    p.exp_string("Delete task").expect("delete confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("delete footer");
    p.exp_eof().expect("delete eof");
}

// Picks an option from inquire's `Select` list (Yes/No/All/Quit ordering).
// Sent as N down-arrows + Enter; the LineWriter requires explicit flush
// because `\r` is not a newline.
pub fn select_option(p: &mut PtySession, choice: &str) {
    let down_count = match choice {
        "Yes" => 0,
        "No" => 1,
        "All" => 2,
        "Quit" => 3,
        _ => panic!("unknown select option: {choice}"),
    };
    for _ in 0..down_count {
        p.send(SELECT_DOWN).expect("send down arrow");
    }
    p.send("\r").expect("send enter");
    p.flush().expect("flush");
}

pub fn assert_pty_exit(p: &mut PtySession, expected_code: i32) {
    p.exp_eof().expect("eof");
    match p.process().wait().expect("wait") {
        WaitStatus::Exited(_, code) => assert_eq!(
            code, expected_code,
            "expected exit {expected_code}, got {code}"
        ),
        other => panic!("expected exit {expected_code}, got {other:?}"),
    }
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
