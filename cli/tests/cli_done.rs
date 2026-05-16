mod common;

use common::{
    assert_no_pending_tasks, assert_pty_exit, dawn_pty, delete_via_pty, extract_uuid, run_stdout,
    select_option,
};
use predicates::{
    prelude::PredicateBooleanExt,
    str::{contains, is_empty},
};

// ── Group A: Pre-filter route ──

// dawn 1 done
// Completed task 1 'buy milk'.
// Completed 1 task.
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

// dawn <prefix> done
// Completed task 1 'buy milk'.
// Completed 1 task.
#[test]
fn done_by_pre_uuid_completes_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];

    common::execute_dawn(&db)
        .args([prefix, "done"])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

// dawn "buy" done
// (Skipped action prompt assertion since index-description matches are not stable)
// Completed 1 task.
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

// dawn 1,2 done
// This command will alter 2 tasks.
// Completed task 1 'one'.
// Completed task 2 'two'.
// Completed 2 tasks.
#[test]
fn done_pre_set_filter_two_tasks_both_completed() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1,2", "done"])
        .assert()
        .success()
        .stdout(
            contains("This command will alter 2 tasks.")
                .and(contains("Completed task 1"))
                .and(contains("Completed task 2"))
                .and(contains("Completed 2 tasks.")),
        );

    assert_no_pending_tasks(&db);
}

// dawn 1-2 done
// This command will alter 2 tasks.
// Completed task 1 'one'.
// Completed task 2 'two'.
// Completed 2 tasks.
#[test]
fn done_by_pre_range_completes_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1-2", "done"])
        .assert()
        .success()
        .stdout(
            contains("This command will alter 2 tasks.")
                .and(contains("Completed task 1"))
                .and(contains("Completed task 2"))
                .and(contains("Completed 2 tasks.")),
        );

    assert_no_pending_tasks(&db);
}

// ── Group B: Promotion route ──

// dawn done 1
// Completed task 1 'buy milk'.
// Completed 1 task.
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

// dawn done <prefix>
// Completed task 1 'buy milk'.
// Completed 1 task.
#[test]
fn done_promotes_uuid_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];

    common::execute_dawn(&db)
        .args(["done", prefix])
        .assert()
        .success()
        .stdout("Completed task 1 'buy milk'.\nCompleted 1 task.\n");

    assert_no_pending_tasks(&db);
}

// dawn done 1,2
// This command will alter 2 tasks.
// Completed task 1 'one'.
// Completed task 2 'two'.
// Completed 2 tasks.
#[test]
fn done_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["done", "1,2"])
        .assert()
        .success()
        .stdout(
            contains("This command will alter 2 tasks.")
                .and(contains("Completed task 1"))
                .and(contains("Completed task 2"))
                .and(contains("Completed 2 tasks.")),
        );

    assert_no_pending_tasks(&db);
}

// ── Group C: Errors / no-op ──

// dawn <prefix> done (on completed task)
// Task <prefix> 'buy milk' is neither pending nor waiting.
// Completed 0 tasks.
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

    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args([prefix, "done"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(
            contains(&format!(
                "Task {prefix} 'buy milk' is neither pending nor waiting."
            ))
            .and(contains("Completed 0 tasks.")),
        );
}

// dawn <prefix> done (on deleted task)
// Task <prefix> 'buy milk' is neither pending nor waiting.
// Completed 0 tasks.
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

    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args([prefix, "done"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(
            contains(&format!(
                "Task {prefix} 'buy milk' is neither pending nor waiting."
            ))
            .and(contains("Completed 0 tasks.")),
        );
}

// dawn <prefix1>,<prefix2> done (one completed, one pending)
// Completed task 1 'beta'.
// Task <prefix1> 'alpha' is neither pending nor waiting.
// Completed 1 task.
#[test]
fn done_mixed_pending_and_completed_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);
    let uid_first = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uid_first[..8];
    let uid_second = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uid_second[..8];
    common::execute_dawn(&db)
        .args([prefix1, "done"])
        .assert()
        .success();

    let target = format!("{prefix1},{prefix2}");
    common::execute_dawn(&db)
        .args([&target, "done"])
        .assert()
        .failure()
        .code(1)
        .stderr(is_empty())
        .stdout(
            contains("This command will alter 2 tasks.")
                .and(contains("is neither pending nor waiting."))
                .and(contains("Completed task 1"))
                .and(contains("Completed 1 task.")),
        );

    assert_no_pending_tasks(&db);
}

// ── Group D: Bulk-confirm route (3+ tasks, per-task Select) ──

// dawn 1,2,3 done
// This command will alter 3 tasks.
// - End will be set to <date>.
// - Status will be changed from 'pending' to 'completed'.
// Complete task 1 'a'? (Yes/No/All/Quit) all
// Completed task 1 'a'.
// Completed task 2 'b'.
// Completed task 3 'c'.
// Completed 3 tasks.
#[test]
fn done_bulk_three_tasks_all_completes_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- End will be set to ").expect("first diff");
    p.exp_string("- Status will be changed from 'pending' to 'completed'.")
        .expect("first diff");
    p.exp_string("Complete task 1").expect("first prompt");
    select_option(&mut p, "All");
    for i in 0..3 {
        p.exp_string(&format!("Completed task {}", i + 1))
            .expect("action line");
    }
    p.exp_string("Completed 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}

// dawn 1,2,3 done
// This command will alter 3 tasks.
#[test]
fn done_bulk_no_skips_one_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- End will be set to ").expect("first diff");
    p.exp_string("- Status will be changed from 'pending' to 'completed'.")
        .expect("first diff");
    p.exp_string("Complete task 1").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Completed task 1").expect("first action");

    p.exp_string("- End will be set to ").expect("second diff");
    p.exp_string("- Status will be changed from 'pending' to 'completed'.")
        .expect("second diff");
    p.exp_string("Complete task 2").expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not completed.")
        .expect("not-completed msg");

    p.exp_string("- End will be set to ").expect("third diff");
    p.exp_string("- Status will be changed from 'pending' to 'completed'.")
        .expect("third diff");
    p.exp_string("Complete task 3").expect("third prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Completed task").expect("third action");
    p.exp_string("Completed 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 1);

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("1 task"));
}

// dawn 1,2,3 done
// This command will alter 3 tasks.
// - End will be set to <date>.
// - Status will be changed from 'pending' to 'completed'.
// Complete task 1 'a'? (Yes/No/All/Quit) yes
// Completed task 1 'a'.
// - End will be set to <date>.
// - Status will be changed from 'pending' to 'completed'.
// Complete task 2 'b'? (Yes/No/All/Quit) quit
// Task not completed.
// Completed 1 task.
#[test]
fn done_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["a", "b", "c"]);

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- End will be set to ").expect("first diff");
    p.exp_string("- Status will be changed from 'pending' to 'completed'.")
        .expect("first diff");
    p.exp_string("Complete task 1").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Completed task 1").expect("first action");

    p.exp_string("- End will be set to ").expect("second diff");
    p.exp_string("- Status will be changed from 'pending' to 'completed'.")
        .expect("second diff");
    p.exp_string("Complete task 2").expect("second prompt");
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

// ── Group E: No matching tasks (NoSpecified) ──

// dawn 99 done
// No tasks specified.
#[test]
fn done_nonexistent_index_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "only"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["99", "done"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("only"));
}

// dawn b074ae01 done
// No tasks specified.
#[test]
fn done_nonexistent_uuid_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["b074ae01", "done"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

// ── Group F: Empty-filter confirmation (TTY) ──

// dawn done
// This command has no filter, ...Are you sure? (y/N) → sent N
// Command prevented from running.
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

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// dawn done
// This command has no filter, ...Are you sure? (y/N) → sent Y
// This command will alter 2 tasks.
// Completed task 1 'one'.
// Completed task 2 'two'.
// Completed 2 tasks.
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
    for i in 0..2 {
        p.exp_string(&format!("Completed task {}", i + 1))
            .expect("action line");
    }
    p.exp_string("Completed 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    assert_no_pending_tasks(&db);
}
