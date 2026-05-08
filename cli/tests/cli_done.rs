mod common;

use common::{
    assert_no_pending_tasks, assert_pty_exit, dawn_pty, delete_via_pty, drain_pty_and_assert_exit,
    extract_uuid, run_stdout, select_option,
};
use predicates::{
    prelude::PredicateBooleanExt,
    str::{contains, is_empty},
};

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

    common::execute_dawn(&db)
        .args(["buy", "done"])
        .assert()
        .success()
        .stdout(contains("Completed 1 task."));

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("fix bug").and(contains("buy milk").not()));
}

#[test]
fn done_pre_set_filter_two_tasks_both_completed() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1,2", "done"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Completed 2 tasks.")));

    assert_no_pending_tasks(&db);
}

#[test]
fn done_by_pre_range_completes_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1-2", "done"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Completed 2 tasks.")));

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

    common::execute_dawn(&db)
        .args(["done", "1,2"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Completed 2 tasks.")));

    assert_no_pending_tasks(&db);
}

// ── Group C: Errors / no-op ──

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

    common::execute_dawn(&db)
        .args([&uuid, "done"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(
            contains("'buy milk' is neither pending nor waiting.")
                .and(contains("Completed 0 tasks.")),
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

    common::execute_dawn(&db)
        .args([&uuid, "done"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(
            contains("'buy milk' is neither pending nor waiting.")
                .and(contains("Completed 0 tasks.")),
        );
}

#[test]
fn done_mixed_pending_and_completed_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uid_first = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uid_second = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    common::execute_dawn(&db)
        .args([&uid_first, "done"])
        .assert()
        .success();

    let target = format!("{uid_first},{uid_second}");
    common::execute_dawn(&db)
        .args([&target, "done"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(
            contains("This command will alter 2 tasks.")
                .and(contains("is neither pending nor waiting."))
                .and(contains("Completed 1 task.")),
        );

    assert_no_pending_tasks(&db);
}

// ── Group D: Bulk-confirm route (3+ tasks, per-task Select) ──

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

    assert_no_pending_tasks(&db);
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

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("1 task"));
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

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("2 tasks"));
}

// Quit prints "Task not completed." for the Quit-task only and breaks; tasks
// after Quit are neither prompted nor printed. Locks the count invariant.
#[test]
fn done_bulk_quit_emits_not_completed_exactly_once() {
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

    let trailing = drain_pty_and_assert_exit(&mut p, 1);
    assert_eq!(
        trailing.matches("Task not completed.").count(),
        1,
        "Quit should emit not-completed exactly once: {trailing}"
    );
    assert_eq!(
        trailing.matches("Completed task").count(),
        0,
        "no action line should fire after Quit: {trailing}"
    );
    assert!(
        trailing.contains("Completed 1 task."),
        "footer should report 1 completion: {trailing}"
    );
}

// done's bulk-confirm diff DOES show "End will be set" — opposite of
// modify --status completed. This anchors the cross-command UX split.
#[test]
fn done_bulk_shows_end_will_be_set_diff_line() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    let (prelude, _) = p
        .exp_regex("Complete task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("End will be set to "),
        "done's diff should include 'End will be set' line: {prelude}"
    );
    assert!(
        prelude.contains("Status will be changed from 'pending' to 'completed'."),
        "done's diff should also include status change line: {prelude}"
    );
    select_option(&mut p, "All");
    p.exp_string("Completed 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
}
