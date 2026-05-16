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

// dawn 1 delete
// Delete task 1 'buy milk'? (y/n) y
// Deleting task 1 'buy milk'.
// Deleted 1 task.
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

// dawn <prefix> delete
// Delete task 1 'buy milk'? (y/n) y
// Deleting task 1 'buy milk'.
// Deleted 1 task.
#[test]
fn delete_by_pre_uuid_deletes_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];

    let mut p = dawn_pty(&db, &[prefix, "delete"]);

    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_no_pending_tasks(&db);
}

// dawn "buy" delete
// Delete task 1 'buy milk'? (y/n) y
// Deleting task 1 'buy milk'.
// Deleted 1 task.
#[test]
fn delete_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    let mut p = dawn_pty(&db, &["buy", "delete"]);

    p.exp_string("Delete task").expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleting task").expect("action line");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("fix bug").and(contains("buy milk").not()));
}

// dawn 1,2 delete
// This command will alter 2 tasks.
// Delete task 1 'one'? (Yes/No/All/Quit) Yes
// Deleting task 1 'one'.
// Delete task 2 'two'? (Yes/No/All/Quit) Yes
// Deleting task 2 'two'.
// Deleted 2 tasks.
#[test]
fn delete_pre_set_filter_two_tasks_both_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["1,2", "delete"]);

    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task 1").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task 1").expect("first action");
    p.exp_string("Delete task 2").expect("second prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task 2").expect("second action");
    p.exp_string("Deleted 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_no_pending_tasks(&db);
}

// dawn 1-2 delete
// This command will alter 2 tasks.
// Delete task 1 'one'? (Yes/No/All/Quit) Yes
// Deleting task 1 'one'.
// Delete task 2 'two'? (Yes/No/All/Quit) Yes
// Deleting task 2 'two'.
// Deleted 2 tasks.
#[test]
fn delete_by_pre_range_two_tasks_both_deleted() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["1-2", "delete"]);

    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task 1").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task 1").expect("first action");
    p.exp_string("Delete task 2").expect("second prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task").expect("second action");
    p.exp_string("Deleted 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_no_pending_tasks(&db);
}

// dawn 1-2,3 delete
// This command will alter 3 tasks.
// Delete task 1 'a'? (Yes/No/All/Quit) All
// Deleting task 1 'a'.
// Deleting task 2 'b'.
// Deleting task 3 'c'.
// Deleted 3 tasks.
#[test]
fn delete_pre_set_with_range_and_index() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c", "d"]);

    let mut p = dawn_pty(&db, &["1-2,3", "delete"]);

    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Delete task 1").expect("first prompt");
    select_option(&mut p, "All");
    for i in 0..3 {
        p.exp_string(&format!("Deleting task {}", i + 1))
            .expect("action line");
    }
    p.exp_string("Deleted 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("1 task"));
}

// ── Group B: Promotion route ──

// dawn delete 1
// Delete task 1 'buy milk'? (y/n) y
// Deleting task 1 'buy milk'.
// Deleted 1 task.
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

// dawn delete <prefix>
// Delete task 1 'buy milk'? (y/n) y
// Deleting task 1 'buy milk'.
// Deleted 1 task.
#[test]
fn delete_promotes_uuid_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];

    let mut p = dawn_pty(&db, &["delete", prefix]);

    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// dawn delete 1,2
// This command will alter 2 tasks.
// Delete task 1 'one'? (Yes/No/All/Quit) Yes
// Deleting task 1 'one'.
// Delete task 2 'two'? (Yes/No/All/Quit) Yes
// Deleting task 2 'two'.
// Deleted 2 tasks.
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

// ── Group C: Errors / no-op ──

// dawn <prefix> delete
// Task <prefix> 'buy milk' is not deletable.
// Deleted 0 tasks.
#[test]
fn delete_already_deleted_task_skipped_partial() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];
    delete_via_pty(&db, prefix);

    common::execute_dawn(&db)
        .args([prefix, "delete"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(contains(&format!(
            "Task {prefix} 'buy milk' is not deletable.\nDeleted 0 tasks."
        )));
}

// dawn <prefix1>,<prefix2> delete
// This command will alter 2 tasks.
// Task <prefix1> 'alpha' is not deletable.
// Delete task <prefix2> 'beta'? (y/n) y
// Deleting task <prefix2> 'beta'.
// Deleted 1 task.
#[test]
fn delete_mixed_pending_and_deleted_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uuid_first = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid_first[..8];
    let uuid_second = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uuid_second[..8];
    delete_via_pty(&db, prefix1);

    let target = format!("{prefix1},{prefix2}");
    let mut p = dawn_pty(&db, &[&target, "delete"]);

    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header counts pre-filter");
    p.exp_string("is not deletable.")
        .expect("not-deletable warning for already-deleted task");
    // Single candidate after filtering → yes/no Confirm path, not Select.
    p.exp_string("Delete task 1")
        .expect("single-confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleting task 1").expect("action line");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);
    assert_no_pending_tasks(&db);
}

// dawn 1 delete
// Delete task 1 'buy milk'? (y/n) n
// Task not deleted.
// Deleted 0 tasks.
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
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// dawn 1,2 delete
// This command will alter 2 tasks.
// Delete task 1 'one'? (Yes/No/All/Quit) Yes
// Deleting task 1 'one'.
// Delete task 2 'two'? (Yes/No/All/Quit) No
// Task not deleted.
// Deleted 1 task.
#[test]
fn delete_bulk_no_skips_one_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b"]);

    let mut p = dawn_pty(&db, &["1,2", "delete"]);

    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task 1").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task 1").expect("first action");

    p.exp_string("Delete task 2").expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not deleted.").expect("not-deleted msg");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("1 task"));
}

// dawn 1,2 delete
// This command will alter 2 tasks.
// Delete task 1 'one'? (Yes/No/All/Quit) Yes
// Deleting task 1 'one'.
// Delete task 2 'two'? (Yes/No/All/Quit) Quit
// Task not deleted.
// Deleted 1 task.
#[test]
fn delete_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b"]);

    let mut p = dawn_pty(&db, &["1,2", "delete"]);

    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    p.exp_string("Delete task 1").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Deleting task 1").expect("first action");

    p.exp_string("Delete task 2").expect("second prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Task not deleted.").expect("not-deleted msg");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("1 task"));
}

// Deleting a completed task succeeds silently: status transitions to deleted,
// stderr is empty (no TW-style "Note: Modified task X is completed..." footnote).
// This intentional divergence treats `delete` as a deliberate transition; the
// footnote stays only on `modify` where the user's intent is ambiguous.
#[test]
fn delete_completed_task_succeeds_without_footnote() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args([prefix, "done"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &[prefix, "delete"]);
    p.exp_string(&format!("Delete task {prefix} 'buy milk'?"))
        .expect("delete prompt");
    p.send_line("y").expect("send y");

    let trailing = drain_pty_and_assert_exit(&mut p, 0);
    assert!(
        trailing.contains("Deleted 1 task."),
        "footer should report 1 deletion: {trailing}"
    );
    assert_eq!(
        trailing.matches("Note: Modified task").count(),
        0,
        "delete on completed task must not emit footnote: {trailing}"
    );

    let info_after = run_stdout(common::execute_dawn(&db).arg(&uuid));
    assert!(
        info_after.contains("Deleted"),
        "info Status row should be Deleted: {info_after}"
    );
}

// dawn delete
// This command has no filter, ... Are you sure? (y/N) n
// Command prevented from running.
#[test]
fn delete_empty_filter_declined_aborts() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["delete"]);
    p.exp_string("Are you sure?")
        .expect("empty-filter confirm prompt");
    p.send_line("n").expect("send n");
    assert_pty_exit(&mut p, 2);

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// dawn delete
// This command has no filter, ... Are you sure? (y/N) y
// Delete task 1 'buy milk'? (y/n) y
// Deleting task 1 'buy milk'.
// Deleted 1 task.
#[test]
fn delete_empty_filter_confirmed_deletes_all_pending() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["delete"]);
    p.exp_string("Are you sure?")
        .expect("empty-filter confirm prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Delete task 1 'buy milk'?")
        .expect("delete prompt");
    p.send_line("y").expect("send y");
    p.exp_string("Deleted 1 task.").expect("footer");
    assert_pty_exit(&mut p, 0);
    assert_no_pending_tasks(&db);
}

// dawn nomatch delete
// No tasks specified.
#[test]
fn delete_filter_matches_no_tasks_no_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["nomatch", "delete"])
        .assert()
        .failure()
        .code(1)
        .stdout(is_empty())
        .stderr(contains("No tasks specified."));
}
