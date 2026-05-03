mod common;

use common::{delete_via_pty, extract_uuid, run_stdout};

#[test]
fn info_single_index_renders_all_base_rows() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let out = common::execute_dawn(&db).arg("1").output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    for header in [
        "ID",
        "Description",
        "Status",
        "Entered",
        "Last modified",
        "UUID",
    ] {
        assert!(stdout.contains(header), "missing row {header}: {stdout}");
    }
    assert!(stdout.contains("buy milk"), "missing description: {stdout}");
    assert!(stdout.contains("Pending"), "missing status: {stdout}");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn info_omits_end_and_deleted_rows_for_pending() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let out = common::execute_dawn(&db).arg("1").output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(!stdout.contains("End"), "unexpected End row: {stdout}");
    assert!(
        !stdout.contains("Deleted"),
        "unexpected Deleted row: {stdout}"
    );
}

#[test]
fn info_completed_task_renders_end_row_and_completed_status() {
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

    let out = common::execute_dawn(&db).arg(&uuid).output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.contains("End"), "missing End row: {stdout}");
    assert!(
        stdout.contains("Completed"),
        "missing Completed status: {stdout}"
    );
    assert!(
        !stdout.contains("Deleted"),
        "unexpected Deleted row/status: {stdout}"
    );
}

#[test]
fn info_deleted_task_renders_deleted_row_and_deleted_status() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    let info_before = run_stdout(common::execute_dawn(&db).arg("1"));
    let uuid = extract_uuid(&info_before);

    delete_via_pty(&db, &uuid);

    let out = common::execute_dawn(&db).arg(&uuid).output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    // "Deleted" appears twice: once as Status value, once as row label.
    assert_eq!(
        stdout.matches("Deleted").count(),
        2,
        "expected Deleted to appear as both status value and row label: {stdout}"
    );
    assert!(!stdout.contains("End"), "unexpected End row: {stdout}");
    assert!(
        !stdout.contains("Completed"),
        "unexpected Completed status: {stdout}"
    );
}

#[test]
fn info_completed_then_deleted_task_renders_both_end_and_deleted_rows() {
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
    delete_via_pty(&db, &uuid);

    let out = common::execute_dawn(&db).arg(&uuid).output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.contains("End"), "missing End row: {stdout}");
    // Terminal status is "Deleted"; row label "Deleted" also rendered.
    assert_eq!(
        stdout.matches("Deleted").count(),
        2,
        "expected Deleted to appear as both status value and row label: {stdout}"
    );
    assert!(
        !stdout.contains("Completed"),
        "unexpected Completed status (deleted is terminal): {stdout}"
    );
}

#[test]
fn info_multiple_bare_args_renders_each_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);
    let out = common::execute_dawn(&db)
        .args(["1", "2"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
}

#[test]
fn info_nonexistent_index_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .arg("99")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn info_nonexistent_uuid_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .arg("00000000-0000-0000-0000-000000000099")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

// ── Taskwarrior-style dispatch: any bare id/uuid → info with merged filter ──

#[test]
fn mixed_set_and_bare_resolves_to_info_with_merged_ids() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);
    let out = common::execute_dawn(&db)
        .args(["1,2", "3"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");

    // Taskwarrior rule: presence of bare "3" dispatches to info.
    // All three ids are merged via UNION, so every task renders as an info table.
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert!(stdout.contains("three"), "missing 'three': {stdout}");

    // Info renders each task's "Last modified" row; three tasks → three occurrences.
    let last_modified_count = stdout.matches("Last modified").count();
    assert_eq!(
        last_modified_count, 3,
        "expected 3 info tables, got {last_modified_count}: {stdout}"
    );

    // No next footer should appear — info path is status-agnostic and does not
    // print the "N tasks" summary.
    assert!(
        !stdout.contains("3 tasks") && !stdout.contains("2 tasks"),
        "unexpected next footer: {stdout}"
    );
}

#[test]
fn non_id_bare_routes_to_next_with_word_filter() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "investigate flaky build"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    // "investigate" parses as neither Index nor UUID, so has_bare_id stays
    // false and the command resolves to `next` with a words filter rather than
    // `info`. Only the matching task renders.
    let out = common::execute_dawn(&db)
        .arg("investigate")
        .output()
        .expect("run");
    assert!(out.status.success(), "expected next path to succeed");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("investigate flaky build"),
        "next table missing matching task: {stdout}"
    );
    assert!(
        !stdout.contains("buy milk"),
        "next table should not include non-matching task: {stdout}"
    );
    assert!(stdout.contains("1 task"), "missing next footer: {stdout}");
}

// Range tokens do not set has_bare_id, but a bare ID alongside a range still
// dispatches to info; the range merges into the Info filter.
#[test]
fn bare_index_with_range_routes_to_info_with_merged_filter() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);
    let out = common::execute_dawn(&db)
        .args(["1", "2-3"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");

    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert!(stdout.contains("three"), "missing 'three': {stdout}");

    let last_modified_count = stdout.matches("Last modified").count();
    assert_eq!(
        last_modified_count, 3,
        "expected 3 info tables, got {last_modified_count}: {stdout}"
    );

    assert!(
        !stdout.contains("3 tasks") && !stdout.contains("2 tasks"),
        "unexpected next footer: {stdout}"
    );
}

#[test]
fn bare_with_nonexistent_id_and_set_filter_exits_cleanly() {
    // Regression: previously, `dawn 1,2 99` triggered both next (1,2) and info (99),
    // producing a partial-failure state (stdout had next output + stderr had
    // "No matches." + exit 1). After the refactor, this resolves to a single info
    // call with merged filter {1, 2, 99}. Tasks 1 and 2 exist, so info succeeds.
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);
    let out = common::execute_dawn(&db)
        .args(["1,2", "99"])
        .output()
        .expect("run");
    assert!(out.status.success(), "expected success via merged info");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert_eq!(stdout.matches("Last modified").count(), 2);
}
