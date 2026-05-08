mod common;

use common::{
    assert_pty_exit, dawn_pty, delete_via_pty, drain_pty_and_assert_exit, extract_uuid, run_stdout,
    select_option,
};
use predicates::{prelude::PredicateBooleanExt, str::contains};

// ── Group A: Pre-filter route ──

#[test]
fn modify_by_pre_index_updates_description() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["1", "modify", "pick", "up", "milk"])
        .assert()
        .success()
        .stdout("Modifying task 1 'pick up milk'.\nModified 1 task.\n");

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(contains("pick up milk").and(contains("buy milk").not()));
}

#[test]
fn modify_by_pre_uuid_updates_description() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    common::execute_dawn(&db)
        .args([&uuid, "modify", "new", "desc"])
        .assert()
        .success()
        .stdout("Modifying task 1 'new desc'.\nModified 1 task.\n");

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("new desc").and(contains("buy milk").not()));
}

#[test]
fn modify_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    common::execute_dawn(&db)
        .args(["buy", "modify", "pick", "up", "milk"])
        .assert()
        .success()
        .stdout(contains("Modified 1 task."));

    common::execute_dawn(&db).assert().success().stdout(
        contains("pick up milk")
            .and(contains("fix bug"))
            .and(contains("buy milk").not()),
    );
}

#[test]
fn modify_pre_set_filter_two_tasks_both_updated() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1,2", "modify", "same"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Modified 2 tasks.")));

    common::execute_dawn(&db).assert().success().stdout(
        contains("same")
            .count(2)
            .and(contains("one").not())
            .and(contains("two").not()),
    );
}

#[test]
fn modify_by_pre_range_updates_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    common::execute_dawn(&db)
        .args(["1-2", "modify", "renamed"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Modified 2 tasks.")));

    common::execute_dawn(&db).assert().success().stdout(
        contains("renamed")
            .count(2)
            .and(contains("alpha").not())
            .and(contains("beta").not()),
    );
}

#[test]
fn modify_pre_filter_with_id_shaped_mod_joins_into_description() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1", "modify", "2", "foo"])
        .assert()
        .success()
        .stdout("Modifying task 1 '2 foo'.\nModified 1 task.\n");

    common::execute_dawn(&db).assert().success().stdout(
        contains("2 foo")
            .and(contains("one").or(contains("two")))
            .and(contains("2 tasks")),
    );
}

// ── Group B: Promotion route ──

#[test]
fn modify_promotes_single_index_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["modify", "1", "pick", "up", "milk"])
        .assert()
        .success()
        .stdout("Modifying task 1 'pick up milk'.\nModified 1 task.\n");

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(contains("pick up milk").and(contains("buy milk").not()));
}

#[test]
fn modify_promotes_uuid_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    common::execute_dawn(&db)
        .args(["modify", &uuid, "new", "desc"])
        .assert()
        .success()
        .stdout("Modifying task 1 'new desc'.\nModified 1 task.\n");

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("new desc").and(contains("buy milk").not()));
}

#[test]
fn modify_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["modify", "1,2", "same"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Modified 2 tasks.")));

    common::execute_dawn(&db).assert().success().stdout(
        contains("same")
            .count(2)
            .and(contains("one").not())
            .and(contains("two").not()),
    );
}

#[test]
fn modify_promotes_range_from_mods() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["modify", "1-2", "same"])
        .assert()
        .success()
        .stdout(contains("This command will alter 2 tasks.").and(contains("Modified 2 tasks.")));

    common::execute_dawn(&db).assert().success().stdout(
        contains("same")
            .count(2)
            .and(contains("one").not())
            .and(contains("two").not()),
    );
}

#[test]
fn modify_promotion_with_id_only_mods_is_noop() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["modify", "1"])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(contains("one"));
}

// `modify --status completed` on an already-completed task is a no-op for the
// timestamp: IFNULL preserves the original `completed` value, and
// `has_changes` filters the task out when no other field differs.
#[test]
fn modify_status_completed_idempotent_on_already_completed_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args([&uuid, "modify", "--status", "completed"])
        .assert()
        .success()
        .stdout(contains("Modified 0 tasks."));
}

// ── --status persistence (round-trip via DB) ──

// `modify --status completed` flips a pending task to completed: info shows
// `Completed` with an `End` row, and the pending list is empty.
#[test]
fn modify_status_completed_persists_pending_to_completed() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));

    common::execute_dawn(&db)
        .args(["1", "modify", "--status", "completed"])
        .assert()
        .success()
        .stdout(contains("Modified 1 task."));

    let info_after = run_stdout(common::execute_dawn(&db).arg(&uuid));
    assert!(
        info_after.contains("Completed"),
        "info Status row should be Completed: {info_after}"
    );
    assert!(
        info_after.contains("End"),
        "info should include End row: {info_after}"
    );

    common::assert_no_pending_tasks(&db);

    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let row = all_view
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row in all view");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[1], "C", "all view status column: {row}");
}

// `modify --status pending` resurrects a completed task: info no longer shows
// an `End` row and the task reappears in the pending list.
#[test]
fn modify_status_pending_persists_completed_to_pending() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args([&uuid, "modify", "--status", "pending"])
        .assert()
        .success()
        .stdout(contains("Modified 1 task."));

    let info_after = run_stdout(common::execute_dawn(&db).arg(&uuid));
    assert!(
        info_after.contains("Pending"),
        "info Status row should be Pending: {info_after}"
    );
    assert!(
        !info_after.contains("End "),
        "info should not include End row: {info_after}"
    );

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// `modify --status deleted` flips a pending task to deleted: info Status row
// reads `Deleted`, the all view shows `D`, and the pending list is empty.
#[test]
fn modify_status_deleted_persists_pending_to_deleted() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));

    common::execute_dawn(&db)
        .args(["1", "modify", "--status", "deleted"])
        .assert()
        .success()
        .stdout(contains("Modified 1 task."));

    let info_after = run_stdout(common::execute_dawn(&db).arg(&uuid));
    assert!(
        info_after.contains("Deleted"),
        "info Status row should be Deleted: {info_after}"
    );

    common::assert_no_pending_tasks(&db);

    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let row = all_view
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row in all view");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[1], "D", "all view status column: {row}");
}

// `modify --status pending` resurrects a deleted task: info no longer shows
// an `End` row and the task reappears in the pending list.
#[test]
fn modify_status_pending_persists_deleted_to_pending() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    common::execute_dawn(&db)
        .args([&uuid, "modify", "--status", "pending"])
        .assert()
        .success()
        .stdout(contains("Modified 1 task."));

    let info_after = run_stdout(common::execute_dawn(&db).arg(&uuid));
    assert!(
        info_after.contains("Pending"),
        "info Status row should be Pending: {info_after}"
    );
    assert!(
        !info_after.contains("End "),
        "info should not include End row: {info_after}"
    );

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// modify --status deleted has no "<col> will be set" line analogous to `done`
// (only the status line). The bulk-confirm diff must announce the status
// change but emit nothing about the deleted timestamp.
#[test]
fn modify_bulk_status_deleted_diff_announces_status_only() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "--status", "deleted"]);
    let (prelude, _) = p
        .exp_regex("Modify task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("Status will be changed from 'pending' to 'deleted'."),
        "diff should announce status change: {prelude}"
    );
    assert!(
        !prelude.contains("End will be set"),
        "modify --status deleted must not print 'End will be set' line: {prelude}"
    );
    assert!(
        !prelude.contains("Deleted will be set"),
        "modify --status deleted must not print 'Deleted will be set' line: {prelude}"
    );
    select_option(&mut p, "All");
    p.exp_string("Modified 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    for desc in ["alpha", "beta", "gamma"] {
        let row = all_view
            .lines()
            .find(|l| l.contains(desc))
            .unwrap_or_else(|| panic!("missing row for {desc}: {all_view}"));
        let cols: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(cols[1], "D", "{desc} should be deleted: {row}");
    }
}

// ── Modify on completed/deleted tasks ──

#[test]
fn modify_completed_task_by_uuid_emits_note() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args([&uuid, "modify", "renamed"])
        .assert()
        .success()
        .stdout(contains("'renamed'").and(contains("Modified 1 task.")))
        .stderr(contains(format!(
            "Note: Modified task {prefix} is completed."
        )));

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("renamed").and(contains("buy milk").not()));
}

#[test]
fn modify_deleted_task_by_uuid_emits_note() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];
    delete_via_pty(&db, &uuid);

    common::execute_dawn(&db)
        .args([&uuid, "modify", "renamed"])
        .assert()
        .success()
        .stdout(contains("'renamed'").and(contains("Modified 1 task.")))
        .stderr(contains(format!(
            "Note: Modified task {prefix} is deleted."
        )));

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("renamed").and(contains("buy milk").not()));
}

// ── Empty-filter prompt (TTY) ──

#[test]
fn modify_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let mut p = dawn_pty(&db, &["modify", "renamed"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("n").expect("send n");
    p.exp_string("Command prevented from running.")
        .expect("abort msg");
    assert_pty_exit(&mut p, 2);

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(contains("buy milk").and(contains("renamed").not()));
}

#[test]
fn modify_no_filter_tty_accept_modifies_all() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let mut p = dawn_pty(&db, &["modify", "renamed"]);
    p.exp_string("This command has no filter")
        .expect("empty-filter prompt");
    p.send_line("y").expect("send y");
    p.exp_string("This command will alter 2 tasks.")
        .expect("alter header");
    for _ in 0..2 {
        p.exp_string("Modifying task").expect("action line");
    }
    p.exp_string("Modified 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    common::execute_dawn(&db).assert().success().stdout(
        contains("renamed")
            .count(2)
            .and(contains("one").not())
            .and(contains("two").not()),
    );
}

// ── Group C: No-op (0-count) ──

#[test]
fn modify_with_pre_filter_but_empty_mods_is_noop() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["1", "modify"])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(contains("one"));
}

#[test]
fn modify_with_whitespace_only_mods_is_noop() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["1", "modify", "   "])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(contains("one"));
}

#[test]
fn modify_to_same_description_is_noop() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["1", "modify", "buy", "milk"])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");
}

// ── Group D: Errors ──

#[test]
fn modify_nonexistent_index_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["99", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

// Out-of-bounds range must not silently mutate; covers the mutation SQL path.
#[test]
fn modify_by_pre_out_of_bounds_range_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "only"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["99-100", "modify", "renamed"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("only").and(contains("renamed").not()));
}

#[test]
fn modify_nonexistent_uuid_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["00000000-0000-0000-0000-000000000099", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

// ── Group E: Bulk-confirm route (3+ tasks, per-task Select) ──

#[test]
fn modify_bulk_three_tasks_all_modified() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Description will be changed from")
        .expect("first diff");
    p.exp_string("Modify task").expect("first prompt");
    select_option(&mut p, "All");
    for _ in 0..3 {
        p.exp_string("Modifying task").expect("action line");
    }
    p.exp_string("Modified 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    common::execute_dawn(&db).assert().success().stdout(
        contains("renamed")
            .count(3)
            .and(contains("alpha").not())
            .and(contains("beta").not())
            .and(contains("gamma").not()),
    );
}

#[test]
fn modify_bulk_no_skips_one_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Modify task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task").expect("first action");
    p.exp_string("Modify task").expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");
    p.exp_string("Modify task").expect("third prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task").expect("third action");
    p.exp_string("Modified 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert_eq!(
        next.matches("renamed").count(),
        2,
        "expected 2 renamed: {next}"
    );
    let untouched = ["alpha", "beta", "gamma"]
        .iter()
        .filter(|w| next.contains(*w))
        .count();
    assert_eq!(untouched, 1, "expected 1 untouched task: {next}");
}

#[test]
fn modify_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Modify task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task").expect("first action");
    p.exp_string("Modify task").expect("second prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");
    p.exp_string("Modified 1 task.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert_eq!(
        next.matches("renamed").count(),
        1,
        "expected 1 renamed: {next}"
    );
    let untouched = ["alpha", "beta", "gamma"]
        .iter()
        .filter(|w| next.contains(*w))
        .count();
    assert_eq!(untouched, 2, "expected 2 untouched tasks: {next}");
}

// ── Group F: Bulk-confirm footnote on completed/deleted attempts ──
//
// Footnote semantics mirror Taskwarrior: emitted per *attempted* task in the
// bulk loop, not only per persisted change. No-answered tasks still emit, and
// the Quit-answered task itself emits — only candidates the loop never reached
// (post-Quit) are silent.

#[test]
fn modify_bulk_no_emits_footnote_for_skipped_completed() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    for uuid in [&uuid1, &uuid2, &uuid3] {
        common::execute_dawn(&db)
            .args([uuid.as_str(), "done"])
            .assert()
            .success();
    }

    let target = format!("{uuid1},{uuid2},{uuid3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Modify task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task").expect("first action");
    p.exp_string("Modify task").expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");
    p.exp_string("Modify task").expect("third prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task").expect("third action");
    p.exp_string("Modified 2 tasks.").expect("footer");

    let trailing = drain_pty_and_assert_exit(&mut p, 1);
    let footnote_count = trailing.matches("Note: Modified task").count();
    assert_eq!(
        footnote_count, 3,
        "expected footnote for every attempted task (incl. No-skipped): {trailing}"
    );
}

#[test]
fn modify_bulk_quit_emits_footnote_through_quit_task_only() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    for uuid in [&uuid1, &uuid2, &uuid3] {
        common::execute_dawn(&db)
            .args([uuid.as_str(), "done"])
            .assert()
            .success();
    }

    let target = format!("{uuid1},{uuid2},{uuid3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Modify task").expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task").expect("first action");
    p.exp_string("Modify task").expect("second prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");
    p.exp_string("Modified 1 task.").expect("footer");

    let trailing = drain_pty_and_assert_exit(&mut p, 1);
    let footnote_count = trailing.matches("Note: Modified task").count();
    assert_eq!(
        footnote_count, 2,
        "expected footnote for Yes-task and Quit-task only, not the unreached candidate: {trailing}"
    );
}

#[test]
fn modify_bulk_explicit_status_suppresses_footnote_even_with_attempts() {
    // The status-explicit guard takes precedence over attempted semantics.
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    for uuid in [&uuid1, &uuid2, &uuid3] {
        common::execute_dawn(&db)
            .args([uuid.as_str(), "done"])
            .assert()
            .success();
    }

    let target = format!("{uuid1},{uuid2},{uuid3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "--status", "pending"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("Modify task").expect("first prompt");
    select_option(&mut p, "All");
    p.exp_string("Modified 3 tasks.").expect("footer");

    let trailing = drain_pty_and_assert_exit(&mut p, 0);
    assert!(
        !trailing.contains("Note: Modified task"),
        "explicit --status must suppress the footnote: {trailing}"
    );
}

// modify --status completed sets the timestamp internally but should NOT show
// the "End will be set" diff line — Taskwarrior's `task modify status:completed`
// only displays the status diff. `task done` is the command that owns that line.
#[test]
fn modify_bulk_status_completed_omits_end_will_be_set_line() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "--status", "completed"]);
    let (prelude, _) = p
        .exp_regex("Modify task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("Status will be changed from 'pending' to 'completed'."),
        "diff should still announce status change: {prelude}"
    );
    assert!(
        !prelude.contains("End will be set"),
        "modify --status completed must not print 'End will be set' line: {prelude}"
    );
    select_option(&mut p, "All");
    p.exp_string("Modified 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
}
