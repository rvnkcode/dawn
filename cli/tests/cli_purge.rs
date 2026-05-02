mod common;

use common::{assert_pty_exit, dawn_pty, delete_via_pty, extract_uid, run_stdout, select_option};
use std::path::Path;

// `task all` body containment check; tolerates the empty-DB exit-1 / stderr
// "No matches." case by returning false rather than panicking.
fn all_contains(db: &Path, description: &str) -> bool {
    let out = common::dawn_cmd(db).arg("all").output().expect("run all");
    String::from_utf8_lossy(&out.stdout).contains(description)
}

fn assert_all_empty(db: &Path) {
    let out = common::dawn_cmd(db).arg("all").output().expect("run all");
    assert_eq!(out.status.code(), Some(1), "expected empty all → exit 1");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("No matches."),
        "expected 'No matches.' stderr: {stderr}"
    );
}

// ── Group A: Pre-filter route (filter before subcommand) ──

#[test]
fn purge_by_pre_uid_purges_deleted_task() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let uid = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    delete_via_pty(&db, &uid);

    let mut p = dawn_pty(&db, &[&uid, "purge"]);
    p.exp_string("Permanently remove task")
        .expect("single confirm prompt");
    p.exp_string("'buy milk'?").expect("description in prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_all_empty(&db);
}

#[test]
fn purge_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    // Capture UID of "buy milk" before deleting; index↔description mapping is unstable.
    let info1 = run_stdout(common::dawn_cmd(&db).arg("1"));
    let info2 = run_stdout(common::dawn_cmd(&db).arg("2"));
    let buy_uid = if info1.contains("buy milk") {
        extract_uid(&info1)
    } else {
        extract_uid(&info2)
    };
    delete_via_pty(&db, &buy_uid);

    let mut p = dawn_pty(&db, &["buy", "purge"]);
    p.exp_string("Permanently remove task")
        .expect("single confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert!(all_contains(&db, "fix bug"), "untouched task missing");
    assert!(!all_contains(&db, "buy milk"), "purged task still present");
}

#[test]
fn purge_pre_set_two_uids_both_purged() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    let uid1 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    let uid2 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("2")));
    delete_via_pty(&db, &uid1);
    delete_via_pty(&db, &uid2);

    // original_count == 2 > 1 → bulk Select fires per task.
    let target = format!("{uid1},{uid2}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);
    p.exp_string("Permanently remove task")
        .expect("first bulk prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Permanently remove task")
        .expect("second bulk prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Purged 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert!(!all_contains(&db, "alpha"), "alpha still present");
    assert!(!all_contains(&db, "beta"), "beta still present");
}

// ── Group B: Empty-filter prompt (TTY) ──

#[test]
fn purge_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    // Decline path returns from `confirm_empty_filter()` before any task lookup,
    // so a pending task is enough to verify the purge was not executed.
    let mut p = dawn_pty(&db, &["purge"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Command prevented from running.")
        .expect("abort msg");
    assert_pty_exit(&mut p, 2);

    assert!(
        all_contains(&db, "buy milk"),
        "task unexpectedly purged after decline"
    );
}

#[test]
fn purge_no_filter_tty_accept_purges_only_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    // Delete via word filter so we know exactly which task is deleted.
    delete_via_pty(&db, "alpha");

    // tasks.len() == 2, deleted.len() == 1 → bulk Select fires once.
    let mut p = dawn_pty(&db, &["purge"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Permanently remove task")
        .expect("bulk select prompt");
    p.exp_string("'alpha'?")
        .expect("alpha must be the deleted candidate");
    select_option(&mut p, "Yes");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert!(!all_contains(&db, "alpha"), "alpha still present");
    assert!(all_contains(&db, "beta"), "beta unexpectedly removed");
}

// ── Group C: Filter resolves to nothing (`tasks.is_empty()`) ──

#[test]
fn purge_no_match_returns_no_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let out = common::dawn_cmd(&db)
        .args(["nonexistentword", "purge"])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(1),
        "expected exit 1 from NoSpecified"
    );
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("No tasks specified."),
        "missing NoSpecified stderr: {stderr}"
    );
}

// ── Group D: Filter matches only pending (`deleted.is_empty()`) ──

#[test]
fn purge_filter_matches_only_pending_prints_yellow() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    // Non-empty filter matches a pending task; deleted set is empty → yellow exit 0.
    let out = common::dawn_cmd(&db)
        .args(["buy", "purge"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0), "expected exit 0");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("No deleted tasks specified."),
        "missing yellow message: {stderr}"
    );
    assert!(all_contains(&db, "buy milk"), "pending task removed");
}

// ── Group E: User confirmation declines / bulk Select branches ──

#[test]
fn purge_user_declines_single_no_op() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uid = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    delete_via_pty(&db, &uid);

    let mut p = dawn_pty(&db, &[&uid, "purge"]);
    p.exp_string("Permanently remove task")
        .expect("single confirm prompt");
    p.send_line("n").expect("send n");
    // Purge has no Partial — declining still exits 0 with "Purged 0 tasks.".
    p.exp_string("Purged 0 tasks.").expect("zero-count footer");
    assert_pty_exit(&mut p, 0);

    assert!(all_contains(&db, "buy milk"), "task purged despite decline");
}

#[test]
fn purge_bulk_no_skips_one() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    let uid1 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    let uid2 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("2")));
    delete_via_pty(&db, &uid1);
    delete_via_pty(&db, &uid2);

    let target = format!("{uid1},{uid2}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);
    p.exp_string("Permanently remove task")
        .expect("first bulk prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Permanently remove task")
        .expect("second bulk prompt");
    select_option(&mut p, "No");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    // Exactly one of the two descriptions must remain (deleted-but-not-purged).
    let alpha = all_contains(&db, "alpha");
    let beta = all_contains(&db, "beta");
    assert!(
        alpha ^ beta,
        "expected exactly one survivor: alpha={alpha} beta={beta}"
    );
}

#[test]
fn purge_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let uid1 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    let uid2 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("2")));
    let uid3 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("3")));
    delete_via_pty(&db, &uid1);
    delete_via_pty(&db, &uid2);
    delete_via_pty(&db, &uid3);

    let target = format!("{uid1},{uid2},{uid3}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);
    p.exp_string("Permanently remove task")
        .expect("first bulk prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Permanently remove task")
        .expect("second bulk prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    // Quit aborts the remaining 2 → 1 purged, 2 deleted-but-not-purged.
    let survivors = ["alpha", "beta", "gamma"]
        .iter()
        .filter(|d| all_contains(&db, d))
        .count();
    assert_eq!(survivors, 2, "expected 2 survivors after Quit");
}

#[test]
fn purge_bulk_all_purges_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let uid1 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    let uid2 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("2")));
    let uid3 = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("3")));
    delete_via_pty(&db, &uid1);
    delete_via_pty(&db, &uid2);
    delete_via_pty(&db, &uid3);

    let target = format!("{uid1},{uid2},{uid3}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);
    p.exp_string("Permanently remove task")
        .expect("first bulk prompt");
    select_option(&mut p, "All");
    p.exp_string("Purged 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_all_empty(&db);
}
