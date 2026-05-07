mod common;

use std::path::Path;

use common::{assert_pty_exit, dawn_pty, delete_via_pty, extract_uuid, run_stdout, select_option};
use predicates::{prelude::PredicateBooleanExt, str::contains};

fn assert_all_empty(db: &Path) {
    common::execute_dawn(db)
        .arg("all")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No matches."));
}

// ── Group A: Pre-filter route (filter before subcommand) ──

#[test]
fn purge_by_pre_uuid_purges_deleted_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    let mut p = dawn_pty(&db, &[&uuid, "purge"]);

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
    // Capture UUID of "buy milk" before deleting; index↔description mapping is unstable.
    let info1 = run_stdout(common::execute_dawn(&db).arg("1"));
    let info2 = run_stdout(common::execute_dawn(&db).arg("2"));
    let buy_uid = if info1.contains("buy milk") {
        extract_uuid(&info1)
    } else {
        extract_uuid(&info2)
    };
    delete_via_pty(&db, &buy_uid);

    let mut p = dawn_pty(&db, &["buy", "purge"]);

    p.exp_string("Permanently remove task")
        .expect("single confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);
    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("fix bug").and(contains("buy milk").not()));
}

#[test]
fn purge_pre_set_two_uids_both_purged() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    delete_via_pty(&db, &uuid1);
    delete_via_pty(&db, &uuid2);

    // original_count == 2 > 1 → bulk Select fires per task.
    let target = format!("{uuid1},{uuid2}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);

    p.exp_string("Permanently remove task")
        .expect("first bulk prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Permanently remove task")
        .expect("second bulk prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Purged 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_all_empty(&db);
}

// ── Group B: Empty-filter prompt (TTY) ──

#[test]
fn purge_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["purge"]);

    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Command prevented from running.")
        .expect("abort msg");
    assert_pty_exit(&mut p, 2);
    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

#[test]
fn purge_no_filter_tty_accept_purges_only_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    // Delete via word filter so we know exactly which task is deleted.
    delete_via_pty(&db, "alpha");

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
    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("alpha").not().and(contains("beta")));
}

// ── Group C: Filter resolves to nothing (`tasks.is_empty()`) ──

#[test]
fn purge_no_match_returns_no_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["nonexistentword", "purge"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No tasks specified."));
}

// ── Group D: Filter matches only pending (`deleted.is_empty()`) ──

#[test]
fn purge_filter_matches_only_pending_prints_yellow() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    // Non-empty filter matches a pending task; deleted set is empty → yellow exit 0.
    common::execute_dawn(&db)
        .args(["buy", "purge"])
        .assert()
        .success()
        .stderr(contains("No deleted tasks specified."));

    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// ── Group E: User confirmation declines / bulk Select branches ──

#[test]
fn purge_user_declines_single_no_op() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    let mut p = dawn_pty(&db, &[&uuid, "purge"]);

    p.exp_string("Permanently remove task")
        .expect("single confirm prompt");
    p.send_line("n").expect("send n");
    // Purge has no Partial — declining still exits 0 with "Purged 0 tasks.".
    p.exp_string("Purged 0 tasks.").expect("zero-count footer");
    assert_pty_exit(&mut p, 0);
    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

#[test]
fn purge_bulk_no_skips_one() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    delete_via_pty(&db, &uuid1);
    delete_via_pty(&db, &uuid2);

    let target = format!("{uuid1},{uuid2}");
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
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let alpha = all_view.contains("alpha");
    let beta = all_view.contains("beta");
    assert!(
        alpha ^ beta,
        "expected exactly one survivor: alpha={alpha} beta={beta}"
    );
}

#[test]
fn purge_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    delete_via_pty(&db, &uuid1);
    delete_via_pty(&db, &uuid2);
    delete_via_pty(&db, &uuid3);

    let target = format!("{uuid1},{uuid2},{uuid3}");
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
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let survivors = ["alpha", "beta", "gamma"]
        .iter()
        .filter(|d| all_view.contains(*d))
        .count();
    assert_eq!(survivors, 2, "expected 2 survivors after Quit");
}

#[test]
fn purge_bulk_all_purges_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    delete_via_pty(&db, &uuid1);
    delete_via_pty(&db, &uuid2);
    delete_via_pty(&db, &uuid3);

    let target = format!("{uuid1},{uuid2},{uuid3}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);

    p.exp_string("Permanently remove task")
        .expect("first bulk prompt");
    select_option(&mut p, "All");
    p.exp_string("Purged 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_all_empty(&db);
}
