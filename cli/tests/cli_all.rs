mod common;

use common::{delete_via_pty, extract_uid, run_stdout};

// Returns the status letter (P/C/D) found in the body row containing `description`.
// Body rows start with the ID column (numeric for pending, "-" for completed/deleted)
// followed by the St column. Header ("ID"/"St") and footer ("N task[s]") rows are
// rejected by the cols[0]/cols[1] shape check.
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

// ── A. Footer / empty ──

#[test]
fn all_with_no_tasks_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .arg("all")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn all_with_one_task_prints_singular_footer() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let stdout = run_stdout(common::dawn_cmd(&db).arg("all"));
    assert!(stdout.contains("buy milk"), "missing description: {stdout}");
    assert!(
        stdout.contains("1 task"),
        "missing singular footer: {stdout}"
    );
    assert!(
        !stdout.contains("1 tasks"),
        "stdout has plural form: {stdout}"
    );
}

#[test]
fn all_with_multiple_tasks_prints_plural_footer() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    let stdout = run_stdout(common::dawn_cmd(&db).arg("all"));
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert!(
        stdout.contains("2 tasks"),
        "missing plural footer: {stdout}"
    );
}

// ── B. Status visibility (pending / completed / deleted) ──

#[test]
fn all_renders_pending_with_index() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let stdout = run_stdout(common::dawn_cmd(&db).arg("all"));
    assert_eq!(status_for(&stdout, "buy milk"), 'P');
    let row = stdout
        .lines()
        .find(|l| l.contains("buy milk"))
        .expect("desc row");
    let cols: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(cols[0], "1", "pending task should expose its index: {row}");
}

#[test]
fn all_renders_completed_with_dash_id() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    common::dawn_cmd(&db).args(["1", "done"]).assert().success();

    let stdout = run_stdout(common::dawn_cmd(&db).arg("all"));
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
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uid = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    delete_via_pty(&db, &uid);

    let stdout = run_stdout(common::dawn_cmd(&db).arg("all"));
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

    // Capture two UIDs by index before mutating state. After `done`/`delete`
    // the pending indices renumber, so anything past this point must address
    // tasks by UID.
    let uid_first = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    let uid_second = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("2")));

    common::dawn_cmd(&db)
        .args([&uid_first, "done"])
        .assert()
        .success();
    delete_via_pty(&db, &uid_second);

    let stdout = run_stdout(common::dawn_cmd(&db).arg("all"));
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

// ── C. Filter pass-through (pre + post merge) ──

// "one"/"two"/"three" cannot be reused here: AllRow's "Done" header column
// always contains the substring "one", which would yield a false positive
// against `stdout.contains("one")` in the post-filter row count below.
// Pick descriptions that share no substring with any AllRow header
// (ID, St, UID, Age, Done, Description).
#[test]
fn all_pre_index_filters_to_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["1", "all"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 1, "expected 1 of 3 tasks: {stdout}");
    assert!(stdout.contains("1 task"), "missing footer: {stdout}");
}

#[test]
fn all_post_index_filters_to_one_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["all", "1"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 1, "expected 1 of 3 tasks: {stdout}");
    assert!(stdout.contains("1 task"), "missing footer: {stdout}");
}

#[test]
fn all_pre_and_post_indices_merge_into_union() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["1", "all", "2"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

#[test]
fn all_set_filter_returns_two_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana", "cherry"]);

    let stdout = run_stdout(common::dawn_cmd(&db).args(["all", "1,2"]));
    let present = ["apple", "banana", "cherry"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

// Words from pre and post merge into a single AND-joined filter
// (build_words_clause joins FTS MATCH terms with AND), unlike index merging
// which is a UNION.
#[test]
fn all_pre_and_post_words_merge_into_and_filter() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "buy bread"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "make milk"])
        .assert()
        .success();

    let stdout = run_stdout(common::dawn_cmd(&db).args(["buy", "all", "milk"]));
    assert!(stdout.contains("buy milk"), "missing AND match: {stdout}");
    assert!(
        !stdout.contains("buy bread"),
        "OR match leaked from 'buy': {stdout}"
    );
    assert!(
        !stdout.contains("make milk"),
        "OR match leaked from 'milk': {stdout}"
    );
    assert!(stdout.contains("1 task"), "missing footer: {stdout}");
}

// ── D. status × filter interactions (where `all` diverges from `next`) ──

#[test]
fn all_word_filter_matches_across_statuses() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "shared keyword one"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "shared keyword two"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "unrelated"])
        .assert()
        .success();

    // Complete the task at index 1 — `next` would hide it, but `all` must
    // still surface it under a word filter.
    common::dawn_cmd(&db).args(["1", "done"]).assert().success();

    let stdout = run_stdout(common::dawn_cmd(&db).args(["all", "shared"]));
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
fn all_uid_filter_matches_deleted_task() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let uid = extract_uid(&run_stdout(common::dawn_cmd(&db).arg("1")));
    delete_via_pty(&db, &uid);

    // Index is gone after deletion — UID is the only handle.
    let stdout = run_stdout(common::dawn_cmd(&db).args(["all", &uid]));
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
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::dawn_cmd(&db)
        .args(["all", "99"])
        .assert()
        .code(1)
        .stderr("No matches.\n");
}
