mod common;

use common::{assert_pty_exit, dawn_pty, delete_via_pty, extract_uid, run_stdout, select_option};

// ── Group A: Pre-filter route ──

#[test]
fn modify_by_pre_index_updates_description() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["1", "modify", "pick", "up", "milk"])
        .assert()
        .success()
        .stdout("Modifying task 1 'pick up milk'.\nModified 1 task.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(
        info.contains("pick up milk"),
        "info missing new description: {info}"
    );
    assert!(
        !info.contains("buy milk"),
        "info still has old description: {info}"
    );
}

#[test]
fn modify_by_pre_uid_updates_description() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    common::dawn_cmd(&db)
        .args([&uid, "modify", "new", "desc"])
        .assert()
        .success()
        .stdout(format!(
            "Modifying task 1 '{}'.\nModified 1 task.\n",
            "new desc"
        ));

    let info_after = run_stdout(common::dawn_cmd(&db).arg(&uid));
    assert!(info_after.contains("new desc"));
    assert!(!info_after.contains("buy milk"));
}

#[test]
fn modify_by_pre_word_filter_matches_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["buy milk", "fix bug"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["buy", "modify", "pick", "up", "milk"]));
    assert!(stdout.contains("Modified 1 task."));

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert!(
        next.contains("pick up milk"),
        "next missing modified task: {next}"
    );
    assert!(
        next.contains("fix bug"),
        "next missing untouched task: {next}"
    );
    assert!(
        !next.contains("buy milk"),
        "next still has old description: {next}"
    );
}

#[test]
fn modify_pre_set_filter_two_tasks_both_updated() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["1,2", "modify", "same"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Modified 2 tasks."));

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert_eq!(
        next.matches("same").count(),
        2,
        "expected both tasks renamed to 'same': {next}"
    );
    assert!(!next.contains("one"));
    assert!(!next.contains("two"));
}

#[test]
fn modify_by_pre_range_updates_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["1-2", "modify", "renamed"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Modified 2 tasks."));

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert_eq!(
        next.matches("renamed").count(),
        2,
        "expected both tasks renamed: {next}"
    );
    assert!(!next.contains("alpha"));
    assert!(!next.contains("beta"));
}

#[test]
fn modify_pre_filter_with_id_shaped_mod_joins_into_description() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::dawn_cmd(&db)
        .args(["1", "modify", "2", "foo"])
        .assert()
        .success()
        .stdout("Modifying task 1 '2 foo'.\nModified 1 task.\n");

    // Index↔description mapping is not stable (see common::setup_tasks).
    // The test verifies that mods' "2" was joined into the description (index 1
    // was modified to "2 foo") rather than treated as a filter for index 2.
    // If "2" had been a filter, both tasks would be modified and "one"/"two"
    // would both disappear.
    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert!(next.contains("2 foo"), "task 1 not renamed: {next}");
    let untouched_remains = next.contains("one") || next.contains("two");
    assert!(
        untouched_remains,
        "untouched task lost its description: {next}"
    );
    assert!(next.contains("2 tasks"), "expected 2-task footer: {next}");
}

// ── Group B: Promotion route ──

#[test]
fn modify_promotes_single_index_from_mods() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["modify", "1", "pick", "up", "milk"])
        .assert()
        .success()
        .stdout("Modifying task 1 'pick up milk'.\nModified 1 task.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(info.contains("pick up milk"));
    assert!(!info.contains("buy milk"));
}

#[test]
fn modify_promotes_uid_from_mods() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    common::dawn_cmd(&db)
        .args(["modify", &uid, "new", "desc"])
        .assert()
        .success()
        .stdout("Modifying task 1 'new desc'.\nModified 1 task.\n");

    let info_after = run_stdout(common::dawn_cmd(&db).arg(&uid));
    assert!(info_after.contains("new desc"));
    assert!(!info_after.contains("buy milk"));
}

#[test]
fn modify_promotes_set_from_mods_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["modify", "1,2", "same"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Modified 2 tasks."));

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert_eq!(next.matches("same").count(), 2);
    assert!(!next.contains("one"));
    assert!(!next.contains("two"));
}

#[test]
fn modify_promotes_range_from_mods() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["modify", "1-2", "same"]));
    assert!(stdout.contains("This command will alter 2 tasks."));
    assert!(stdout.contains("Modified 2 tasks."));

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert_eq!(next.matches("same").count(), 2);
    assert!(!next.contains("one"));
    assert!(!next.contains("two"));
}

#[test]
fn modify_promotion_with_id_only_mods_is_noop() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["modify", "1"])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(info.contains("one"));
}

// ── Modify on completed/deleted tasks ──

#[test]
fn modify_completed_task_by_uid_emits_note() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    common::dawn_cmd(&db).args(["1", "done"]).assert().success();

    let stdout = run_stdout(common::dawn_cmd(&db).args([&uid, "modify", "renamed"]));
    assert!(
        stdout.contains("'renamed'"),
        "action line missing: {stdout}"
    );
    assert!(
        stdout.contains("Modified 1 task."),
        "footer missing: {stdout}"
    );
    assert!(
        stdout.contains(&format!("Note: Modified task {uid} is completed.")),
        "missing completed note: {stdout}"
    );

    let info_after = run_stdout(common::dawn_cmd(&db).arg(&uid));
    assert!(
        info_after.contains("renamed"),
        "description not updated: {info_after}"
    );
    assert!(
        !info_after.contains("buy milk"),
        "old description remains: {info_after}"
    );
}

#[test]
fn modify_deleted_task_by_uid_emits_note() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::dawn_cmd(&db).arg("1"));
    let uid = extract_uid(&info_before);

    delete_via_pty(&db, &uid);

    let stdout = run_stdout(common::dawn_cmd(&db).args([&uid, "modify", "renamed"]));
    assert!(
        stdout.contains("'renamed'"),
        "action line missing: {stdout}"
    );
    assert!(
        stdout.contains("Modified 1 task."),
        "footer missing: {stdout}"
    );
    assert!(
        stdout.contains(&format!("Note: Modified task {uid} is deleted.")),
        "missing deleted note: {stdout}"
    );

    let info_after = run_stdout(common::dawn_cmd(&db).arg(&uid));
    assert!(
        info_after.contains("renamed"),
        "description not updated: {info_after}"
    );
    assert!(
        !info_after.contains("buy milk"),
        "old description remains: {info_after}"
    );
}

// ── Empty-filter prompt (TTY) ──

#[test]
fn modify_no_filter_tty_decline_aborts() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
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

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(
        info.contains("buy milk"),
        "task unexpectedly changed: {info}"
    );
    assert!(
        !info.contains("renamed"),
        "task unexpectedly changed: {info}"
    );
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

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert_eq!(
        next.matches("renamed").count(),
        2,
        "expected both tasks renamed: {next}"
    );
    assert!(!next.contains("one"));
    assert!(!next.contains("two"));
}

// ── Group C: No-op (0-count) ──

#[test]
fn modify_with_pre_filter_but_empty_mods_is_noop() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["1", "modify"])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(info.contains("one"));
}

#[test]
fn modify_with_whitespace_only_mods_is_noop() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["1", "modify", "   "])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(info.contains("one"));
}

#[test]
fn modify_to_same_description_is_noop() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["1", "modify", "buy", "milk"])
        .assert()
        .success()
        .stdout("Modified 0 tasks.\n");
}

// ── Group D: Errors ──

#[test]
fn modify_nonexistent_index_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["99", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

// Out-of-bounds range must not silently mutate; covers the mutation SQL path.
#[test]
fn modify_by_pre_out_of_bounds_range_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["99-100", "modify", "renamed"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert!(next.contains("only"), "task was mutated: {next}");
    assert!(!next.contains("renamed"), "renamed leaked: {next}");
}

#[test]
fn modify_nonexistent_uid_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["abc1efghijkl", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

// Regression: nanoid SAFE alphabet allows UIDs starting with '-'. Without
// `allow_hyphen_values` on the pre-filter, clap rejects them as unknown flags.
#[test]
fn modify_by_pre_hyphen_prefixed_uid_does_not_panic_clap() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["-Abc1efghijk", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}

#[test]
fn modify_promotion_with_hyphen_prefixed_uid_does_not_panic_clap() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["modify", "-Abc1efghijk", "foo"])
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

    let next = run_stdout(&mut common::dawn_cmd(&db));
    assert_eq!(
        next.matches("renamed").count(),
        3,
        "expected all 3 renamed: {next}"
    );
    for original in ["alpha", "beta", "gamma"] {
        assert!(
            !next.contains(original),
            "old description '{original}' remains: {next}"
        );
    }
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

    let next = run_stdout(&mut common::dawn_cmd(&db));
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

    let next = run_stdout(&mut common::dawn_cmd(&db));
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
