mod common;

use common::{extract_uid, run_stdout};

fn assert_no_pending_tasks(db: &std::path::Path) {
    let out = common::dawn_cmd(db).output().expect("run");
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
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["1", "done"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_by_pre_uid_completes_task() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    common::dawn_cmd(&db)
        .args([&uid, "done"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["buy", "done"]));
    assert!(stdout.contains("Completed 1 task."));

    let next = run_stdout(&mut common::dawn_cmd(&db));
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

    let stdout = run_stdout(common::dawn_cmd(&db).args(["1,2", "done"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Completed 2 tasks."));

    assert_no_pending_tasks(&db);
}

// ── Group B: Promotion route ──

#[test]
fn done_promotes_single_index_from_mods() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["done", "1"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_promotes_uid_from_mods() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    common::dawn_cmd(&db)
        .args(["done", &uid])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

#[test]
fn done_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["done", "1,2"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Completed 2 tasks."));

    assert_no_pending_tasks(&db);
}

// ── Group C: Empty-filter abort (non-TTY) ──

#[test]
fn done_no_filter_aborts_under_non_tty() {
    common::assert_empty_filter_aborts(&["done"]);
}

#[test]
fn done_promotion_word_only_does_not_promote_aborts_under_non_tty() {
    common::assert_empty_filter_aborts(&["done", "text"]);
}

// ── Group D: Errors / no-op ──

#[test]
fn done_nonexistent_index_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["99", "done"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

#[test]
fn done_nonexistent_uid_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["abc1efghijkl", "done"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

#[test]
fn done_already_completed_task_skipped_partial() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    common::dawn_cmd(&db).args(["1", "done"]).assert().success();

    let out = common::dawn_cmd(&db)
        .args([&uid, "done"])
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

// TODO: add `done_already_deleted_task_skipped_partial` once `delete` exists.
