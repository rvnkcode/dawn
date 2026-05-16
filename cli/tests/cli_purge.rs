mod common;

use std::path::Path;

use common::{
    assert_pty_exit, dawn_pty, delete_via_pty, drain_pty_and_assert_exit, extract_uuid, run_stdout,
    select_option,
};
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

// dawn <prefix> purge
// Permanently remove task <prefix> 'buy milk'? (y/n) y
// Purged 1 task.
#[test]
fn purge_by_pre_uuid_purges_deleted_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix = &uuid[..8];
    delete_via_pty(&db, &uuid);

    let mut p = dawn_pty(&db, &[prefix, "purge"]);

    p.exp_string(&format!("Permanently remove task {prefix} 'buy milk'?"))
        .expect("single confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Purged 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_all_empty(&db);
}

// dawn "buy" purge
#[test]
fn purge_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);
    // Capture UUID of "buy milk" before deleting; index↔description mapping is unstable.
    let info1 = run_stdout(common::execute_dawn(&db).arg("1"));
    let info2 = run_stdout(common::execute_dawn(&db).arg("2"));
    let buy_uuid = if info1.contains("buy milk") {
        extract_uuid(&info1)
    } else {
        extract_uuid(&info2)
    };
    let prefix = &buy_uuid[..8];
    delete_via_pty(&db, prefix);

    let mut p = dawn_pty(&db, &["buy", "purge"]);

    p.exp_string(&format!("Permanently remove task {prefix} 'buy milk'?"))
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

// dawn <prefix1>,<prefix2> purge
// Permanently remove task <prefix1> 'alpha'? (y/n) y
// Permanently remove task <prefix2> 'beta'? (y/n) y
// Purged 2 tasks.
#[test]
fn purge_pre_set_two_uuids_both_purged() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix1 = &uuid1[..8];
    let prefix2 = &uuid2[..8];
    delete_via_pty(&db, prefix1);
    delete_via_pty(&db, prefix2);

    // original_count == 2 > 1 → bulk Select fires per task.
    let target = format!("{prefix1},{prefix2}");
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

// ── Group B: Filter resolves to nothing (`tasks.is_empty()`) ──

// dawn "nonexistentword" purge
// No tasks specified.
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

// ── Group C: Filter matches only pending (`deleted.is_empty()`) ──

// dawn "buy" purge
// Purged 0 tasks.
// No deleted tasks specified. Maybe you forgot to delete tasks first?
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
        .stdout(contains("Purged 0 tasks."))
        .stderr(contains(
            "No deleted tasks specified. Maybe you forgot to delete tasks first?",
        ));

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// dawn "buy" purge (on a completed task)
// Purged 0 tasks.
// No deleted tasks specified. Maybe you forgot to delete tasks first?
#[test]
fn purge_filter_matches_only_completed_prints_yellow() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["buy", "purge"])
        .assert()
        .success()
        .stdout(contains("Purged 0 tasks."))
        .stderr(contains("No deleted tasks specified."));

    // Completed task remains in `all` view.
    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// Mixed filter (1 pending + 1 deleted): only the deleted task is offered for
// purge; the pending task is silently skipped (no second prompt).
// dawn <prefix1>,<prefix2> purge
// Permanently remove task <prefix1> 'alpha'? (y/n) y
// Purged 1 task.
#[test]
fn purge_mixed_pending_and_deleted_only_deleted_offered() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix1 = &uuid1[..8];
    let prefix2 = &uuid2[..8];
    delete_via_pty(&db, prefix1);

    let target = format!("{prefix1},{prefix2}");
    let mut p = dawn_pty(&db, &[&target, "purge"]);

    // original_count == 2 → Select widget for the lone deleted candidate.
    p.exp_string(&format!("Permanently remove task {prefix1}"))
        .expect("bulk prompt for deleted candidate");
    select_option(&mut p, "Yes");

    let trailing = drain_pty_and_assert_exit(&mut p, 0);
    assert!(
        trailing.contains("Purged 1 task."),
        "footer should report exactly 1 purge — pending must be skipped: {trailing}"
    );

    // Exactly one survivor (the pending one); the purged uuid is gone.
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    assert!(
        all_view.contains(prefix2) && !all_view.contains(prefix1),
        "expected pending survivor and purged uuid to be gone: {all_view}"
    );
}

// Empty-filter TTY decline: aborts with exit 2; tasks untouched.
#[test]
fn purge_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    let mut p = dawn_pty(&db, &["purge"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Command prevented from running.")
        .expect("abort msg");
    assert_pty_exit(&mut p, 2);

    // Deleted task survives — purge was aborted before reaching the filter.
    common::execute_dawn(&db)
        .arg("all")
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// Empty-filter TTY accept: proceeds to enumerate all tasks; only deleted ones
// are offered for purge.
#[test]
fn purge_no_filter_tty_accept_purges_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid1[..8];
    delete_via_pty(&db, prefix1);

    let mut p = dawn_pty(&db, &["purge"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("y").expect("send y");

    // tasks.len() == 2 → Select widget for the 1 deleted candidate.
    p.exp_string(&format!("Permanently remove task {prefix1}"))
        .expect("bulk prompt for deleted candidate");
    select_option(&mut p, "Yes");

    let trailing = drain_pty_and_assert_exit(&mut p, 0);
    assert!(
        trailing.contains("Purged 1 task."),
        "footer should report exactly 1 purge — pending must be skipped: {trailing}"
    );

    // Exactly one survivor — the pending one.
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let alpha = all_view.contains("alpha");
    let beta = all_view.contains("beta");
    assert!(
        alpha ^ beta,
        "expected exactly one survivor: alpha={alpha} beta={beta}"
    );
}

// ── Group D: User confirmation declines / bulk Select branches ──

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
