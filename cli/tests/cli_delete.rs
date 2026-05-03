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
fn delete_by_pre_index_deletes_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["1", "delete"]);
    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleting task 1 'buy milk'.")
        .expect("action line");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

#[test]
fn delete_by_pre_uuid_deletes_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    let mut p = dawn_pty(&db, &[&uuid, "delete"]);
    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

#[test]
fn delete_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    let mut p = dawn_pty(&db, &["buy", "delete"]);
    p.exp_string("Delete task").expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(
        next.contains("fix bug"),
        "next missing untouched task: {next}"
    );
    assert!(
        !next.contains("buy milk"),
        "deleted task still in next: {next}"
    );
}

#[test]
fn delete_pre_set_filter_two_tasks_both_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["1,2", "delete"]);
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("first action");
    p.exp_string("Delete task").expect("second prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("second action");
    p.exp_string("Deleted 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

#[test]
fn delete_by_pre_range_two_tasks_both_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["1-2", "delete"]);
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("first action");
    p.exp_string("Delete task").expect("second prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("second action");
    p.exp_string("Deleted 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// `1-2,3` selects 3 of 4 seeded tasks → bulk-confirm Select route.
#[test]
fn delete_pre_set_with_range_and_index() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c", "d"]);

    let mut p = dawn_pty(&db, &["1-2,3", "delete"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "All");
    for _ in 0..3 {
        p.exp_string("Deleting task").expect("action line");
    }
    p.exp_string("Deleted 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(next.contains("1 task"), "expected 1 remaining: {next}");
}

#[test]
fn delete_bulk_all_path_deletes_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "delete"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "All");
    // After "All", remaining tasks are batch-confirmed; expect 3 action lines.
    for _ in 0..3 {
        p.exp_string("Deleting task").expect("action line");
    }
    p.exp_string("Deleted 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// ── Group B: Promotion route ──

#[test]
fn delete_promotes_single_index_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["delete", "1"]);
    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

#[test]
fn delete_promotes_uuid_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    let mut p = dawn_pty(&db, &["delete", &uuid]);
    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

#[test]
fn delete_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["delete", "1,2"]);
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("first action");
    p.exp_string("Delete task").expect("second prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("second action");
    p.exp_string("Deleted 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// ── Group C: Empty-filter prompt (TTY) ──

#[test]
fn delete_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["delete"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Command prevented from running.")
        .expect("abort msg");
    assert_pty_exit(&mut p, 2);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(
        next.contains("buy milk"),
        "task unexpectedly deleted: {next}"
    );
}

#[test]
fn delete_no_filter_tty_accept_deletes_all() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["delete"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("y").expect("send y");
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "All");
    for _ in 0..2 {
        p.exp_string("Deleting task").expect("action line");
    }
    p.exp_string("Deleted 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// ── Group D: Errors / no-op ──

#[test]
fn delete_already_deleted_task_skipped_partial() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    delete_via_pty(&db, &uuid);

    let out = common::execute_dawn(&db)
        .args([&uuid, "delete"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1), "expected Partial exit 1");
    assert!(out.stderr.is_empty(), "Partial should not write stderr");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("'buy milk' is not deletable."),
        "missing not-deletable warning: {stdout}"
    );
    assert!(
        stdout.contains("Deleted 0 tasks."),
        "missing 0-count footer: {stdout}"
    );
}

// Filter resolves to one already-deleted task and one pending task.
// The matched count (2) exceeds the approved count (1) so the command exits
// Partial — matching Taskwarrior's behavior when only a subset of the
// matched tasks could be acted on.
#[test]
fn delete_mixed_pending_and_deleted_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    // alpha/beta ↔ index mapping is non-deterministic — capture both UIDs
    // before the fixture deletion so the test does not depend on it.
    let uid_first = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uid_second = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));

    delete_via_pty(&db, &uid_first);

    let target = format!("{uid_first},{uid_second}");
    let mut p = dawn_pty(&db, &[&target, "delete"]);
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header counts pre-filter");
    p.exp_string("is not deletable.")
        .expect("not-deletable warning for already-deleted task");
    // Single candidate after filtering → yes/no Confirm path, not Select.
    p.exp_string("Delete task").expect("single-confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleting task").expect("action line");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);

    assert_no_pending_tasks(&db);
}

#[test]
fn delete_user_declines_partial() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["1", "delete"]);
    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Task not deleted.").expect("not-deleted msg");
    p.exp_string("Deleted 0 tasks.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(
        next.contains("buy milk"),
        "declined task missing from next: {next}"
    );
}

#[test]
fn delete_bulk_no_skips_one_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b"]);

    let mut p = dawn_pty(&db, &["1,2", "delete"]);
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("first action");
    p.exp_string("Delete task").expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not deleted.").expect("not-deleted msg");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(next.contains("1 task"), "expected 1 remaining: {next}");
}

#[test]
fn delete_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b"]);

    let mut p = dawn_pty(&db, &["1,2", "delete"]);
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("first action");
    p.exp_string("Delete task").expect("second prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Task not deleted.").expect("not-deleted msg");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert!(next.contains("1 task"), "expected 1 remaining: {next}");
}
