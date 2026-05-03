mod common;

use common::{assert_pty_exit, dawn_pty, delete_via_pty, extract_uuid, run_stdout, select_option};

fn assert_no_pending_tasks(db: &std::path::Path) {
    let out = common::execute_dawn(db).output().expect("run");
    assert_eq!(
        out.status.code(),
        Some(1),
        "expected exit 1 from empty next view"
    );
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("No matches."),
        "expected 'No matches.' stderr: {stderr}"
    );
}

// ── Group A: Pre-filter route ──

#[test]
fn done_by_pre_index_completes_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_by_pre_uuid_completes_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    common::execute_dawn(&db)
        .args([&uuid, "done"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["buy", "done"]));
    assert!(stdout.contains("Completed 1 task."));

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(
        next.contains("fix bug"),
        "next missing untouched task: {next}"
    );
    assert!(
        !next.contains("buy milk"),
        "completed task still in next: {next}"
    );
}

#[test]
fn done_pre_set_filter_two_tasks_both_completed() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["1,2", "done"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Completed 2 tasks."));

    assert_no_pending_tasks(&db);
}

#[test]
fn done_by_pre_range_completes_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["1-2", "done"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Completed 2 tasks."));

    assert_no_pending_tasks(&db);
}

// ── Group B: Promotion route ──

#[test]
fn done_promotes_single_index_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["done", "1"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_promotes_uuid_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    common::execute_dawn(&db)
        .args(["done", &uuid])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["done", "1,2"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Completed 2 tasks."));

    assert_no_pending_tasks(&db);
}

// ── Group C: Empty-filter prompt (TTY) ──

#[test]
fn done_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["done"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Command prevented from running.")
        .expect("abort msg");
    assert_pty_exit(&mut p, 2);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(
        next.contains("buy milk"),
        "task unexpectedly completed: {next}"
    );
}

#[test]
fn done_no_filter_tty_accept_completes_all() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["done"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("y").expect("send y");
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    for _ in 0..2 {
        p.exp_string("Completed task").expect("action line");
    }
    p.exp_string("Completed 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// ── Group D: Errors / no-op ──

#[test]
fn done_already_completed_task_skipped_partial() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    let out = common::execute_dawn(&db)
        .args([&uuid, "done"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1), "expected Partial exit 1");
    assert!(out.stderr.is_empty(), "Partial should not write stderr");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("'buy milk' is neither pending nor waiting."),
        "missing skip warning: {stdout}"
    );
    assert!(
        stdout.contains("Completed 0 tasks."),
        "missing 0-count footer: {stdout}"
    );
}

#[test]
fn done_already_deleted_task_skipped_partial() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    delete_via_pty(&db, &uuid);

    let out = common::execute_dawn(&db)
        .args([&uuid, "done"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1), "expected Partial exit 1");
    assert!(out.stderr.is_empty(), "Partial should not write stderr");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("'buy milk' is neither pending nor waiting."),
        "missing skip warning: {stdout}"
    );
    assert!(
        stdout.contains("Completed 0 tasks."),
        "missing 0-count footer: {stdout}"
    );
}

// Filter resolves to one already-completed task and one pending task.
// The matched count (2) exceeds the approved count (1) so the command exits
// Partial — matching Taskwarrior's behavior when only a subset of the
// matched tasks could be acted on. With <3 matches, no per-task prompt fires
// and the surviving candidate is auto-approved.
#[test]
fn done_mixed_pending_and_completed_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    // alpha/beta ↔ index mapping is non-deterministic — capture both UIDs
    // before the fixture completion so the test does not depend on it.
    let uid_first = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uid_second = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));

    common::execute_dawn(&db)
        .args([&uid_first, "done"])
        .assert()
        .success();

    let target = format!("{uid_first},{uid_second}");
    let out = common::execute_dawn(&db)
        .args([&target, "done"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1), "expected Partial exit 1");
    assert!(out.stderr.is_empty(), "Partial should not write stderr");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("This command will alter 2 tasks."),
        "missing pre-filter alter header: {stdout}"
    );
    assert!(
        stdout.contains("is neither pending nor waiting."),
        "missing skip warning: {stdout}"
    );
    assert!(
        stdout.contains("Completed 1 task."),
        "missing 1-count footer: {stdout}"
    );

    assert_no_pending_tasks(&db);
}

// ── Group E: Bulk-confirm route (3+ tasks, per-task Select) ──

#[test]
fn done_bulk_three_tasks_all_completes_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Status will be changed from 'pending' to 'completed'.")
        .expect("first diff");
    p.exp_string("Complete task").expect("first prompt");
    select_option(&mut p, "All");
    for _ in 0..3 {
        p.exp_string("Completed task").expect("action line");
    }
    p.exp_string("Completed 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    let out = common::execute_dawn(&db).output().expect("run");
    assert_eq!(out.status.code(), Some(1), "expected empty next view");
}

#[test]
fn done_bulk_no_skips_one_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Complete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Completed task").expect("first action");
    p.exp_string("Complete task").expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not completed.")
        .expect("not-completed msg");
    p.exp_string("Complete task").expect("third prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Completed task").expect("third action");
    p.exp_string("Completed 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(next.contains("1 task"), "expected 1 remaining: {next}");
}

#[test]
fn done_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Complete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Completed task").expect("first action");
    p.exp_string("Complete task").expect("second prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Task not completed.")
        .expect("not-completed msg");
    p.exp_string("Completed 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(next.contains("2 tasks"), "expected 2 remaining: {next}");
}
