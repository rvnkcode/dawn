mod common;

// extract UID from `info` output
fn extract_uid(stdout: &str) -> String {
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("UID")
            && let Some(token) = rest.split_whitespace().next()
            && token.len() == 12
            && token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return token.to_string();
        }
    }
    panic!("UID row not found in info output:\n{stdout}");
}

fn run_stdout(cmd: &mut assert_cmd::Command) -> String {
    let out = cmd.output().expect("run");
    assert!(
        out.status.success(),
        "command failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

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
fn modify_pre_filter_multiword_description_joined() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "old"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["1", "modify", "pick", "up", "milk"])
        .assert()
        .success()
        .stdout("Modifying task 1 'pick up milk'.\nModified 1 task.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(info.contains("pick up milk"));
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
fn modify_promotes_blank_pre_strings_treated_as_empty() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "old"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["", "modify", "1", "foo"])
        .assert()
        .success()
        .stdout("Modifying task 1 'foo'.\nModified 1 task.\n");

    let info = run_stdout(common::dawn_cmd(&db).arg("1"));
    assert!(info.contains("foo"));
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

fn assert_empty_filter_aborts(args: &[&str]) {
    let (_dir, db) = common::test_db();
    let out = common::dawn_cmd(&db).args(args).output().expect("run");
    assert_eq!(out.status.code(), Some(2), "expected exit 2");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("Command prevented from running."),
        "stderr missing abort message: {stderr}"
    );
}

#[test]
fn modify_promotion_word_only_does_not_promote_aborts_under_non_tty() {
    assert_empty_filter_aborts(&["modify", "text", "modification"]);
}

#[test]
fn modify_promotion_leading_zero_treated_as_word_aborts_under_non_tty() {
    assert_empty_filter_aborts(&["modify", "007", "foo"]);
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

#[test]
fn modify_nonexistent_uid_prints_no_tasks_specified() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["abc1efghijkl", "modify", "foo"])
        .assert()
        .code(1)
        .stderr("No tasks specified.\n");
}
