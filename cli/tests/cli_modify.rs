mod common;

use common::{
    assert_pty_exit, dawn_pty, delete_via_pty, drain_pty_and_assert_exit, extract_uuid, run_stdout,
    select_option,
};
use predicates::{prelude::PredicateBooleanExt, str::contains};

// ── Description modify — pre-filter route ──

// dawn 1 modify pick up milk
// Modifying task 1 'pick up milk'.
// Modified 1 task.
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
        .assert()
        .success()
        .stdout(contains("pick up milk").and(contains("buy milk").not()));
}

// dawn <uuid_prefix> modify new desc
// Modifying task 1 'new desc'.
// Modified 1 task.
#[test]
fn modify_by_pre_uuid_updates_description() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];

    common::execute_dawn(&db)
        .args([prefix, "modify", "new", "desc"])
        .assert()
        .success()
        .stdout("Modifying task 1 'new desc'.\nModified 1 task.\n");

    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("new desc").and(contains("buy milk").not()));
}

// dawn "buy" modify pick up milk
// (skip preview assertion since index-description match is unstable)
// Modified 1 task.
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

// dawn 1,2 modify same
// This command will alter 2 tasks.
// Modifying task 1 'same'.
// Modifying task 2 'same'.
// Modified 2 tasks.
#[test]
fn modify_pre_set_filter_two_tasks_both_updated() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1,2", "modify", "same"])
        .assert()
        .success()
        .stdout("This command will alter 2 tasks.\nModifying task 1 'same'.\nModifying task 2 'same'.\nModified 2 tasks.\n");

    common::execute_dawn(&db).assert().success().stdout(
        contains("same")
            .count(2)
            .and(contains("one").not())
            .and(contains("two").not()),
    );
}

// dawn 1-2 modify renamed
// This command will alter 2 tasks.
// Modifying task 1 'renamed'.
// Modifying task 2 'renamed'.
// Modified 2 tasks.
#[test]
fn modify_by_pre_range_updates_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    common::execute_dawn(&db)
        .args(["1-2", "modify", "renamed"])
        .assert()
        .success()
        .stdout("This command will alter 2 tasks.\nModifying task 1 'renamed'.\nModifying task 2 'renamed'.\nModified 2 tasks.\n");

    common::execute_dawn(&db).assert().success().stdout(
        contains("renamed")
            .count(2)
            .and(contains("alpha").not())
            .and(contains("beta").not()),
    );
}

// dawn 1 modify 2 foo
// Modifying task 1 '2 foo'.
// Modified 1 task.
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

// ── Description modify — promotion route ──

// dawn modify 1 pick up milk
// Modifying task 1 'pick up milk'.
// Modified 1 task.
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

// dawn modify <uuid_prefix> new desc
// Modifying task 1 'new desc'.
// Modified 1 task.
#[test]
fn modify_promotes_uuid_from_mods() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);
    let prefix = &uuid[..8];

    common::execute_dawn(&db)
        .args(["modify", prefix, "new", "desc"])
        .assert()
        .success()
        .stdout("Modifying task 1 'new desc'.\nModified 1 task.\n");

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("new desc").and(contains("buy milk").not()));
}

// dawn modify 1,2 same
// This command will alter 2 tasks.
// Modifying task 1 'same'.
// Modifying task 2 'same'.
// Modified 2 tasks.
#[test]
fn modify_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["modify", "1,2", "same"])
        .assert()
        .success()
        .stdout("This command will alter 2 tasks.\nModifying task 1 'same'.\nModifying task 2 'same'.\nModified 2 tasks.\n");

    common::execute_dawn(&db).assert().success().stdout(
        contains("same")
            .count(2)
            .and(contains("one").not())
            .and(contains("two").not()),
    );
}

// ── No-op (0 tasks modified) ──

// dawn 1 modify
// Modified 0 tasks.
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

// dawn 1 modify "   "
// Modified 0 tasks.
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

// dawn 1 modify buy milk
// Modified 0 tasks.
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

// dawn modify 1
// Modified 0 tasks.
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

// ── Filter errors (No tasks specified) ──

// dawn 99 modify "foo"
// No tasks specified.
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
// dawn 99-100 modify renamed
// No tasks specified.
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

// dawn b074ae01 modify foo
// No tasks specified.
#[test]
fn modify_nonexistent_uuid_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["b074ae01", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

// ── Empty-filter confirmation (TTY) ──

// dawn modify "renamed"
// This command has no filter, ...Are you sure? (y/N) → sent N
// Command prevented from running.
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

// dawn modify "renamed"
// This command has no filter, ...Are you sure? (y/N) → sent Y
// This command will alter 2 tasks.
// Modifying task 1 'renamed'.
// Modifying task 2 'renamed'.
// Modified 2 tasks.
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

// ── Bulk-confirm — 3+ tasks ──

// dawn 1,2,3 modify renamed
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task <id> 'renamed'? (Yes/No/All/Quit) → sent All
// Modifying task 1 'renamed'.
// Modifying task 2 'renamed'.
// Modifying task 3 'renamed'.
// Modified 3 tasks.
#[test]
fn modify_bulk_three_tasks_all_modified() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Description will be changed from")
        .expect("first diff");
    p.exp_string("Modify task 1 'renamed'?")
        .expect("first prompt");
    select_option(&mut p, "All");
    for i in 0..3 {
        p.exp_string(&format!("Modifying task {} 'renamed'.", i + 1))
            .expect(&format!("action for task {}", i + 1));
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

// dawn 1,2,3 modify renamed
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task 1 'renamed'? (Yes/No/All/Quit) → Y
// Modifying task 1 'renamed'.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task 2 'renamed'? (Yes/No/All/Quit) → N
// Task not modified.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task 3 'renamed'? (Yes/No/All/Quit) → Y
// Modifying task 3 'renamed'.
// Modified 2 tasks.
#[test]
fn modify_bulk_no_skips_one_partial() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Description will be changed from")
        .expect("first diff");
    p.exp_string("Modify task 1 'renamed'?")
        .expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task 1 'renamed'.")
        .expect("first action");

    p.exp_string("- Description will be changed from")
        .expect("second diff");
    p.exp_string("Modify task 2 'renamed'?")
        .expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");

    p.exp_string("- Description will be changed from")
        .expect("third diff");
    p.exp_string("Modify task 3 'renamed'?")
        .expect("third prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task 3 'renamed'.")
        .expect("third action");
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

// dawn 1,2,3 modify renamed
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task 1 'renamed'? (Yes/No/All/Quit) → Y
// Modifying task 1 'renamed'.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task 2 'renamed'? (Yes/No/All/Quit) → Quit
// Task not modified.
// Modified 1 task.
#[test]
fn modify_bulk_quit_aborts_remaining() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Description will be changed from")
        .expect("first diff");
    p.exp_string("Modify task 1 'renamed'?")
        .expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string("Modifying task 1 'renamed'.")
        .expect("first action");

    p.exp_string("- Description will be changed from")
        .expect("second diff");
    p.exp_string("Modify task 2 'renamed'?")
        .expect("second prompt");
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

// dawn 1,2,3 modify renamed
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task 1 'renamed'? (Yes/No/All/Quit) → Quit
// Task not modified.
// Modified 0 tasks.
#[test]
fn modify_bulk_quit_on_first_modifies_nothing() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Description will be changed from")
        .expect("first diff");
    p.exp_string("Modify task 1 'renamed'?")
        .expect("first prompt");
    select_option(&mut p, "Quit");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");
    p.exp_string("Modified 0 tasks.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert_eq!(
        next.matches("renamed").count(),
        0,
        "nothing should be renamed: {next}"
    );
    let untouched = ["alpha", "beta", "gamma"]
        .iter()
        .filter(|w| next.contains(*w))
        .count();
    assert_eq!(untouched, 3, "all 3 tasks should be untouched: {next}");
}

// dawn 1,2,3 modify renamed
// This command will alter 3 tasks.
// Modify task 1/2/3 'renamed'? (Yes/No/All/Quit) → No, No, No
// Modified 0 tasks.
#[test]
fn modify_bulk_no_on_all_modifies_nothing() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    for i in 0..3 {
        p.exp_string("- Description will be changed from")
            .expect("diff");
        p.exp_string(&format!("Modify task {} 'renamed'?", i + 1))
            .expect("prompt");
        select_option(&mut p, "No");
        p.exp_string("Task not modified.")
            .expect("not-modified msg");
    }
    p.exp_string("Modified 0 tasks.").expect("footer");
    assert_pty_exit(&mut p, 1);

    let next = run_stdout(&mut common::execute_dawn(&db));
    assert_eq!(
        next.matches("renamed").count(),
        0,
        "nothing should be renamed: {next}"
    );
    let untouched = ["alpha", "beta", "gamma"]
        .iter()
        .filter(|w| next.contains(*w))
        .count();
    assert_eq!(untouched, 3, "all 3 tasks should be untouched: {next}");
}

// Task with no changes should be skipped
// dawn 1,2,3 modify alpha
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'alpha'.
// Modify task 2 'beta'? (Yes/No/All/Quit) → Y
// Modifying task 2 'alpha'.
// - Description will be changed from '<old>' to 'alpha'.
// Modify task 3 'gamma'? (Yes/No/All/Quit) → Y
// Modifying task 3 'alpha'.
// Modified 2 tasks.
#[test]
fn modify_bulk_partial_candidates() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "alpha"]);

    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    for _ in 0..2 {
        p.exp_string("- Description will be changed from")
            .expect("diff");
        p.exp_string("Modify task").expect("prompt");
        select_option(&mut p, "Yes");
        p.exp_string("Modifying task").expect("action");
    }
    p.exp_string("Modified 2 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
    let next = run_stdout(&mut common::execute_dawn(&db));
    assert_eq!(
        next.matches("alpha").count(),
        3,
        "expected 3 renamed: {next}"
    );
}

// ── Completed/deleted footnote — single task ──

// dawn <completed_prefix> modify renamed
// Modifying task <prefix> 'renamed'.
// Modified 1 task.
// Note: Modified task <prefix> is completed. You may wish to make this task pending with: task <prefix> modify
// --status pending
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
        .args([prefix, "modify", "renamed"])
        .assert()
        .success()
        .stdout(
            contains(&format!("Modifying task {prefix} 'renamed'.\nModified 1 task.\n"))
        )
        .stderr(contains(format!(
            "Note: Modified task {prefix} is completed. You may wish to make this task pending with: task {prefix} modify --status pending",
        )));

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("renamed").and(contains("buy milk").not()));
}

// dawn <deleted_prefix> modify renamed
// Modifying task <prefix> 'renamed'.
// Modified 1 task.
// Note: Modified task <prefix> is deleted. You may wish to make this task pending with: task
// <prefix> modify --status pending
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
    delete_via_pty(&db, prefix);

    common::execute_dawn(&db)
        .args([prefix, "modify", "renamed"])
        .assert()
        .success()
        .stdout(
            contains(&format!("Modifying task {prefix} 'renamed'.\nModified 1 task.\n"))
        )
        .stderr(contains(format!(
            "Note: Modified task {prefix} is deleted. You may wish to make this task pending with: task {prefix} modify --status pending",
        )));

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(contains("renamed").and(contains("buy milk").not()));
}

// ── Completed/deleted footnote — bulk-confirm ──

// dawn <prefix1>,<prefix2>,<prefix3> modify "renamed"
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task <prefix1> 'renamed'? (Yes/No/All/Quit) → Y
// Modifying task <prefix1> 'renamed'.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task <prefix2> 'renamed'? (Yes/No/All/Quit) → N
// Task not modified.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task <prefix3> 'renamed'? (Yes/No/All/Quit) → Y
// Modifying task <prefix3> 'renamed'.
// Modified 2 tasks.
// Note: Modified task... printed for all 3 tasks
#[test]
fn modify_bulk_no_emits_footnote_for_skipped_completed() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid1[..8];
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uuid2[..8];
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    let prefix3 = &uuid3[..8];
    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("Complete task").expect("done prompt");
    select_option(&mut p, "All");
    p.exp_string("Completed 3 tasks.").expect("done footer");
    assert_pty_exit(&mut p, 0);

    let target = format!("{prefix1},{prefix2},{prefix3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Description will be changed from")
        .expect("first diff");
    p.exp_string(&format!("Modify task {prefix1} 'renamed'?"))
        .expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string(&format!("Modifying task {prefix1} 'renamed'."))
        .expect("first action");

    p.exp_string("- Description will be changed from")
        .expect("second diff");
    p.exp_string(&format!("Modify task {prefix2} 'renamed'?"))
        .expect("second prompt");
    select_option(&mut p, "No");
    p.exp_string("Task not modified.")
        .expect("not-modified msg");

    p.exp_string("- Description will be changed from")
        .expect("third diff");
    p.exp_string(&format!("Modify task {prefix3} 'renamed'?"))
        .expect("third prompt");
    select_option(&mut p, "Yes");
    p.exp_string(&format!("Modifying task {prefix3} 'renamed'."))
        .expect("third action");
    p.exp_string("Modified 2 tasks.").expect("footer");

    let trailing = drain_pty_and_assert_exit(&mut p, 1);
    let footnote_count = trailing.matches("Note: Modified task").count();
    assert_eq!(
        footnote_count, 3,
        "expected footnote for every attempted task (incl. No-skipped): {trailing}"
    );
}

// dawn <prefix1>,<prefix2>,<prefix3> modify "renamed"
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task <prefix1> 'renamed'? (Yes/No/All/Quit) → Y
// Modifying task <prefix1> 'renamed'.
// - Description will be changed from '<old>' to 'renamed'.
// Modify task <prefix2> 'renamed'? (Yes/No/All/Quit) → Quit
// Task not modified.
// Modified 1 task.
// Note: Modified task... printed for two tasks
#[test]
fn modify_bulk_quit_emits_footnote_through_quit_task_only() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid1[..8];
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uuid2[..8];
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    let prefix3 = &uuid3[..8];
    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("Complete task").expect("done prompt");
    select_option(&mut p, "All");
    p.exp_string("Completed 3 tasks.").expect("done footer");
    assert_pty_exit(&mut p, 0);

    let target = format!("{prefix1},{prefix2},{prefix3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "renamed"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Description will be changed from")
        .expect("first diff");
    p.exp_string(&format!("Modify task {prefix1} 'renamed'?"))
        .expect("first prompt");
    select_option(&mut p, "Yes");
    p.exp_string(&format!("Modifying task {prefix1} 'renamed'."))
        .expect("first action");

    p.exp_string("- Description will be changed from")
        .expect("second diff");
    p.exp_string(&format!("Modify task {prefix2} 'renamed'?"))
        .expect("second prompt");
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

// --status option

// `modify --status pending` resurrects a completed task
//
// dawn <completed_prefix> modify --status pending
// Modifying task <prefix> 'buy milk'.
// Modified 1 task.
#[test]
fn modify_status_pending_persists_completed_to_pending() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args([prefix, "modify", "--status", "pending"])
        .assert()
        .success()
        .stdout(
            contains(&format!("Modifying task {prefix} 'buy milk'."))
                .and(contains("Modified 1 task.")),
        );

    let info_after = run_stdout(common::execute_dawn(&db).arg("1"));
    assert!(
        info_after.contains("Pending"),
        "info Status row should be Pending: {info_after}"
    );
    assert!(
        !info_after.contains("End "),
        "info should not include End row: {info_after}"
    );
    // Assert next table
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// `modify --status pending` resurrects a deleted task
//
// dawn <deleted_prefix> modify --status pending
// Modifying task <prefix> 'buy milk'.
// Modified 1 task.
#[test]
fn modify_status_pending_persists_deleted_to_pending() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args([prefix, "modify", "--status", "pending"])
        .assert()
        .success()
        .stdout(contains(&format!(
            "Modifying task {prefix} 'buy milk'.\nModified 1 task.\n"
        )));

    let info_after = run_stdout(common::execute_dawn(&db).arg(&uuid));
    assert!(
        info_after.contains("Pending"),
        "info Status row should be Pending: {info_after}"
    );
    assert!(
        !info_after.contains("End "),
        "info should not include End row: {info_after}"
    );
    // Assert next table
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

// dawn <prefix1>,<prefix2>,<prefix3> modify --status pending
// This command will alter 3 tasks.
// - Status will be changed from 'completed' to 'pending'.
// Modify task <prefix1> 'alpha'? (Yes/No/All/Quit) → All
// Modifying task <prefix1> 'alpha'.
// Modifying task <prefix2> 'beta'.
// Modifying task <prefix3> 'gamma'.
// Modified 3 tasks.
#[test]
fn modify_bulk_explicit_status_suppresses_footnote_even_with_attempts() {
    // The status-explicit guard takes precedence over attempted semantics.
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid1[..8];
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uuid2[..8];
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    let prefix3 = &uuid3[..8];
    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("Complete task").expect("done prompt");
    select_option(&mut p, "All");
    p.exp_string("Completed 3 tasks.").expect("done footer");
    assert_pty_exit(&mut p, 0);

    let target = format!("{prefix1},{prefix2},{prefix3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "--status", "pending"]);
    p.exp_string("This command will alter 3 tasks.")
        .expect("alter header");
    p.exp_string("- Status will be changed from 'completed' to 'pending'.")
        .expect("status diff");
    p.exp_string(&format!("Modify task {prefix1}"))
        .expect("first prompt");
    select_option(&mut p, "All");
    for prefix in &[prefix1, prefix2, prefix3] {
        p.exp_string(&format!("Modifying task {prefix}"))
            .expect(&format!("action for task {prefix}"));
    }
    p.exp_string("Modified 3 tasks.").expect("footer");

    let trailing = drain_pty_and_assert_exit(&mut p, 0);
    assert!(
        !trailing.contains("Note: Modified task"),
        "explicit --status must suppress the footnote: {trailing}"
    );
}

// dawn 1 modify --status completed
// Modifying task 1 'buy milk'.
// Modified 1 task.
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
        .stdout(contains(&format!(
            "Modifying task 1 'buy milk'.\nModified 1 task.\n"
        )));

    let prefix = &uuid[..8];
    let info_after = run_stdout(common::execute_dawn(&db).arg(prefix));
    assert!(
        info_after.contains("Completed"),
        "info Status row should be Completed: {info_after}"
    );
    assert!(
        info_after.contains("End"),
        "info should include End row: {info_after}"
    );

    common::assert_no_pending_tasks(&db);
    // Assert 'all' table
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let row = all_view
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row in all view");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[1], "C", "all view status column: {row}");
}

// modify --status completed sets the timestamp internally,
// but should NOT show the "End will be set" diff line (only for the `done` command)
#[test]
fn modify_bulk_status_completed_omits_end_will_be_set_line() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(&db, &["1,2,3", "modify", "--status", "completed"]);
    let (prelude, _) = p
        .exp_regex("Modify task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("- Status will be changed from 'pending' to 'completed'."),
        "diff should still announce status change: {prelude}"
    );
    assert!(
        !prelude.contains("- End will be set"),
        "modify --status completed must not print 'End will be set' line: {prelude}"
    );
    select_option(&mut p, "All");
    p.exp_string("Modified 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);
}

// `modify --status completed` on a completed task is a no-op
// dawn <completed_prefix> modify --status completed
// Modified 0 tasks.
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

    let prefix = &uuid[..8];
    common::execute_dawn(&db)
        .args([prefix, "modify", "--status", "completed"])
        .assert()
        .success()
        .stdout(contains("Modified 0 tasks."))
        .stderr(contains(format!(
            "Note: Modified task {prefix} is completed. You may wish to make this task pending with: task {prefix} modify --status pending",
        )));
}

// dawn <p1>,<p2>,<p3> modify --status completed (on deleted tasks)
// This command will alter 3 tasks.
// - Status will be changed from 'deleted' to 'completed'.
// Modify task <p1> 'alpha'? (Yes/No/All/Quit) → All
// Modified 3 tasks.
#[test]
fn modify_bulk_status_completed_diff_on_deleted_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid1[..8];
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uuid2[..8];
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    let prefix3 = &uuid3[..8];

    let mut p = dawn_pty(&db, &["1,2,3", "delete"]);
    p.exp_string("Delete task").expect("delete prompt");
    select_option(&mut p, "All");
    p.exp_string("Deleted 3 tasks.").expect("delete footer");
    assert_pty_exit(&mut p, 0);

    let target = format!("{prefix1},{prefix2},{prefix3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "--status", "completed"]);
    let (prelude, _) = p
        .exp_regex("Modify task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("- Status will be changed from 'deleted' to 'completed'."),
        "diff should announce status change: {prelude}"
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
        assert_eq!(cols[1], "C", "{desc} should be completed: {row}");
    }
}

// dawn 1 modify --status deleted
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
        .stdout(contains("Modifying task 1 'buy milk'.\nModified 1 task."));

    let prefix = &uuid[..8];
    let info_after = run_stdout(common::execute_dawn(&db).arg(prefix));
    assert!(
        info_after.contains("Deleted"),
        "info Status row should be Deleted: {info_after}"
    );
    common::assert_no_pending_tasks(&db);
    // Assert 'all' table
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    let row = all_view
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row in all view");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[1], "D", "all view status column: {row}");
}

// dawn 1,2,3 modify --status deleted
// This command will alter 3 tasks.
// - Status will be changed from 'pending' to 'deleted'.
// Modify task 1 'alpha'? (Yes/No/All/Quit) → All
// Modifying task 1 'alpha'.
// Modifying task 2 'beta'.
// Modifying task 3 'gamma'.
// Modified 3 tasks.
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
    for i in 0..3 {
        p.exp_string(&format!("Modifying task {} '", i + 1))
            .expect(&format!("action for task {}", i + 1));
    }
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

// dawn <p1>,<p2>,<p3> modify --status deleted (on completed tasks)
// This command will alter 3 tasks.
// - Status will be changed from 'completed' to 'deleted'.
// Modify task <p1> 'alpha'? (Yes/No/All/Quit) → All
// Modified 3 tasks.
#[test]
fn modify_bulk_status_deleted_diff_on_completed_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uuid1 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix1 = &uuid1[..8];
    let uuid2 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    let prefix2 = &uuid2[..8];
    let uuid3 = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("3")));
    let prefix3 = &uuid3[..8];

    let mut p = dawn_pty(&db, &["1,2,3", "done"]);
    p.exp_string("Complete task").expect("done prompt");
    select_option(&mut p, "All");
    p.exp_string("Completed 3 tasks.").expect("done footer");
    assert_pty_exit(&mut p, 0);

    let target = format!("{prefix1},{prefix2},{prefix3}");
    let mut p = dawn_pty(&db, &[&target, "modify", "--status", "deleted"]);
    let (prelude, _) = p
        .exp_regex("Modify task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("- Status will be changed from 'completed' to 'deleted'."),
        "diff should announce status change: {prelude}"
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

// dawn <deleted_prefix> modify --status deleted
// Modified 0 tasks.
// Note: Modified task <prefix> is deleted...
#[test]
fn modify_status_idempotent_on_already_deleted_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix = &uuid[..8];
    delete_via_pty(&db, &uuid);

    common::execute_dawn(&db)
        .args([prefix, "modify", "--status", "deleted"])
        .assert()
        .success()
        .stdout(contains("Modified 0 tasks."))
        .stderr(contains(format!(
            "Note: Modified task {prefix} is deleted. You may wish to make this task pending with: task {prefix} modify --status pending",
        )));
}

// dawn <deleted_prefix> modify --status completed
// Modifying task <prefix> 'buy milk'.
// Modified 1 task.
#[test]
fn modify_status_completed_on_deleted_task_without_footnote() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let prefix = &uuid[..8];
    delete_via_pty(&db, &uuid);

    common::execute_dawn(&db)
        .args([prefix, "modify", "--status", "completed"])
        .assert()
        .success()
        .stdout(contains(format!(
            "Modifying task {prefix} 'buy milk'.\nModified 1 task."
        )))
        .stderr(contains(format!("Note: Modified task",)).not());
}

// dawn 1 modify renamed --status completed
// Modifying task 1 'renamed'.
// Modified 1 task.
#[test]
fn modify_description_and_status_together_single_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));

    common::execute_dawn(&db)
        .args(["1", "modify", "renamed", "--status", "completed"])
        .assert()
        .success()
        .stdout(contains("Modifying task 1 'renamed'.\nModified 1 task."));

    let prefix = &uuid[..8];
    let info_after = run_stdout(common::execute_dawn(&db).arg(prefix));
    assert!(
        info_after.contains("renamed"),
        "description should be updated: {info_after}"
    );
    assert!(
        info_after.contains("Completed"),
        "status should be Completed: {info_after}"
    );
    common::assert_no_pending_tasks(&db);
}

// dawn 1,2,3 modify --status completed renamed
// This command will alter 3 tasks.
// - Description will be changed from '<old>' to 'renamed'.
// - Status will be changed from 'pending' to 'completed'.
// Modify task 1 'renamed'? (Yes/No/All/Quit) → All
// Modified 3 tasks.
#[test]
fn modify_bulk_description_and_status_diff_announces_both() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);

    let mut p = dawn_pty(
        &db,
        &["1,2,3", "modify", "--status", "completed", "renamed"],
    );
    let (prelude, _) = p
        .exp_regex("Modify task")
        .expect("first bulk-confirm prompt");
    assert!(
        prelude.contains("- Description will be changed from"),
        "diff should announce description change: {prelude}"
    );
    assert!(
        prelude.contains("- Status will be changed from 'pending' to 'completed'."),
        "diff should announce status change: {prelude}"
    );
    select_option(&mut p, "All");
    p.exp_string("Modified 3 tasks.").expect("footer");
    assert_pty_exit(&mut p, 0);

    common::assert_no_pending_tasks(&db);
    let all_view = run_stdout(common::execute_dawn(&db).arg("all"));
    assert_eq!(
        all_view.matches("renamed").count(),
        3,
        "all three should be renamed: {all_view}"
    );
}
