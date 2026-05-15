mod common;

use common::{delete_via_pty, extract_uuid, run_stdout};
use predicates::{prelude::PredicateBooleanExt, str::contains};

// Pick status column from description
fn status_for(stdout: &str, description: &str) -> char {
    let row = stdout
        .lines()
        .find(|l| l.contains(description))
        .unwrap_or_else(|| panic!("row missing for {description}: {stdout}"));
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert!(cols.len() >= 2, "row too short for {description}: {row:?}");
    assert!(
        cols[0] == "-" || cols[0].chars().all(|c| c.is_ascii_digit()),
        "unexpected ID column for {description}: {row:?}"
    );
    let status = cols[1];
    assert!(
        matches!(status, "P" | "C" | "D"),
        "unexpected status {status:?} in row: {row:?}"
    );
    status.chars().next().unwrap()
}

// ── A. Status visibility (pending / completed / deleted) ──

#[test]
fn all_renders_completed_with_dash_id() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    let stdout = run_stdout(common::execute_dawn(&db).arg("all"));

    assert_eq!(status_for(&stdout, "buy milk"), 'C');
    let row = stdout
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[0], "-", "completed task drops its index: {row}");
}

#[test]
fn all_renders_deleted_with_dash_id() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    let stdout = run_stdout(common::execute_dawn(&db).arg("all"));

    assert_eq!(status_for(&stdout, "buy milk"), 'D');
    let row = stdout
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[0], "-", "deleted task drops its index: {row}");
}

#[test]
fn all_shows_pending_completed_and_deleted_together() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta", "gamma"]);
    let uid_first = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    let uid_second = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("2")));
    common::execute_dawn(&db)
        .args([&uid_first, "done"])
        .assert()
        .success();
    delete_via_pty(&db, &uid_second);

    let stdout = run_stdout(common::execute_dawn(&db).arg("all"));

    assert!(stdout.contains("alpha"), "missing alpha: {stdout}");
    assert!(stdout.contains("beta"), "missing beta: {stdout}");
    assert!(stdout.contains("gamma"), "missing gamma: {stdout}");
    assert!(
        stdout.contains("3 tasks"),
        "missing plural footer: {stdout}"
    );

    // Description↔index mapping is unstable, so collect statuses as a set.
    let mut statuses: Vec<char> = ["alpha", "beta", "gamma"]
        .iter()
        .map(|d| status_for(&stdout, d))
        .collect();
    statuses.sort();
    assert_eq!(statuses, vec!['C', 'D', 'P']);
}

// ── B. Filter pass-through (pre + post merge) ──

// "one"/"two"/"three" cannot be reused here: AllRow's "Done" header column
// always contains the substring "one", which would yield a false positive
// against `stdout.contains("one")` in the post-filter row count below.
// Pick descriptions that share no substring with any AllRow header
// (ID, St, UUID, Age, Done, Description).

// dawn 1 all
// 1 apple
// 1 task
#[test]
fn all_pre_index_filters_to_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["1", "all"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 1, "expected 1 of 3 tasks: {stdout}");
    assert!(stdout.contains("1 task"), "missing footer: {stdout}");
}

// dawn all 1
// 1 apple
// 1 task
#[test]
fn all_post_index_filters_to_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["all", "1"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 1, "expected 1 of 3 tasks: {stdout}");
    assert!(stdout.contains("1 task"), "missing footer: {stdout}");
}

// dawn 1 all 2
// 1 apple
// 2 banana
// 2 tasks
#[test]
fn all_pre_and_post_indices_merge_into_union() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["1", "all", "2"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

// dawn 1-2 all
// 1 apple
// 2 banana
// 2 tasks
#[test]
fn all_pre_range_filters_to_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["1-2", "all"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

// dawn all 1-2
// 1 apple
// 2 banana
// 2 tasks
#[test]
fn all_post_range_filters_to_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["all", "1-2"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

// dawn all 1,2
// 1 apple
// 2 banana
// 2 tasks
#[test]
fn all_set_filter_returns_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::execute_dawn(&db).args(["all", "1,2"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

// Words from pre and post merge into a single AND-joined filter
// dawn buy all milk
// 1 buy milk
// 1 task
#[test]
fn all_pre_and_post_words_merge_into_and_filter() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["add", "buy bread"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["add", "make milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["buy", "all", "milk"])
        .assert()
        .success()
        .stdout(
            contains("buy milk")
                .and(contains("buy bread").not())
                .and(contains("make milk").not())
                .and(contains("1 task")),
        );
}

// ── C. status × filter interactions (where `all` diverges from `next`) ──

#[test]
fn all_word_filter_matches_across_statuses() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "shared keyword one"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["add", "shared keyword two"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["add", "unrelated"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["shared", "keyword", "one", "done"])
        .assert()
        .success();

    let stdout = run_stdout(common::execute_dawn(&db).args(["all", "shared"]));

    assert!(
        stdout.contains("shared keyword one"),
        "first match missing: {stdout}"
    );
    assert!(
        stdout.contains("shared keyword two"),
        "second match missing: {stdout}"
    );
    assert!(
        !stdout.contains("unrelated"),
        "non-matching task leaked: {stdout}"
    );
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");

    // Exactly one of the two surviving rows must be Completed; the other Pending.
    let mut statuses: Vec<char> = ["shared keyword one", "shared keyword two"]
        .iter()
        .map(|d| status_for(&stdout, d))
        .collect();
    statuses.sort();
    assert_eq!(statuses, vec!['C', 'P']);
}

#[test]
fn all_uuid_filter_matches_deleted_task() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uuid = extract_uuid(&run_stdout(common::execute_dawn(&db).arg("1")));
    delete_via_pty(&db, &uuid);

    // Index is gone after deletion — UUID is the only handle.
    let stdout = run_stdout(common::execute_dawn(&db).args(["all", &uuid]));

    assert!(
        stdout.contains("buy milk"),
        "deleted task missing: {stdout}"
    );
    assert_eq!(status_for(&stdout, "buy milk"), 'D');
    assert!(stdout.contains("1 task"), "missing footer: {stdout}");
}

#[test]
fn all_filter_with_no_match_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["all", "99"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No matches."));
}
