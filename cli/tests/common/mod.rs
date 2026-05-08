#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::str::contains;
use rexpect::{
    process::WaitStatus,
    session::{PtySession, spawn_command},
};
use tempfile::TempDir;

const PTY_TIMEOUT_MS: u64 = 5000;
const SELECT_DOWN: &str = "\x1b[B";

// Return `TempDir` to ensure temp dir and DB alive until test end
pub fn test_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    (dir, db)
}

// Executes binary with env var override to use test DB
pub fn execute_dawn(db: &Path) -> Command {
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

// Drain remaining PTY output up to EOF and assert exit code. Returns the
// trailing buffer so callers can count occurrences (e.g. footnote lines).
pub fn drain_pty_and_assert_exit(p: &mut PtySession, expected_code: i32) -> String {
    let trailing = p.exp_eof().expect("eof");
    match p.process().wait().expect("wait") {
        WaitStatus::Exited(_, code) => assert_eq!(
            code, expected_code,
            "expected exit {expected_code}, got {code}"
        ),
        other => panic!("expected exit {expected_code}, got {other:?}"),
    }
    trailing
}

// Tasks share `entry` seconds, so the Index↔description mapping is not stable
// (tiebreaker is the random UUID lex order). Tests must not assume that the
// i-th description receives Index i; filter on all seeded indices, or assert
// on counts rather than on which description maps to which index.
pub fn setup_tasks(db: &Path, descriptions: &[&str]) {
    for desc in descriptions {
        execute_dawn(db).args(["add", desc]).assert().success();
    }
}

// extract UUID from `info` output
pub fn extract_uuid(stdout: &str) -> String {
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("UUID")
            && let Some(token) = rest.split_whitespace().next()
            && uuid::Uuid::parse_str(token).is_ok()
        {
            return token.to_string();
        }
    }
    panic!("UUID row not found in info output:\n{stdout}");
}

// Asserts the default `next` view is empty: exit 1 with "No matches." on stderr.
pub fn assert_no_pending_tasks(db: &Path) {
    execute_dawn(db)
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No matches."));
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
