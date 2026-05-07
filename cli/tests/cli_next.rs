mod common;

use predicates::{prelude::PredicateBooleanExt, str::contains};

#[test]
fn next_with_no_tasks_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn next_with_one_task_prints_singular_footer() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db).assert().success().stdout(
        contains("buy milk")
            .and(contains("1 task"))
            .and(contains("1 tasks").not()),
    );
}

#[test]
fn next_with_multiple_tasks_prints_plural_footer() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["add", "two"])
        .assert()
        .success();

    common::execute_dawn(&db).assert().success().stdout(
        contains("one")
            .and(contains("two"))
            .and(contains("2 tasks")),
    );
}

// ── Filter: set (comma-separated) → next table ──

#[test]
fn next_filter_set_two_indices() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);

    let stdout = common::run_stdout(common::execute_dawn(&db).arg("1,2"));

    // Index↔description mapping is not stable (see common::setup_tasks).
    // Assert that the set filter selects exactly 2 of the 3 seeded tasks.
    let present = ["one", "two", "three"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 2, "expected 2 of 3 tasks to match: {stdout}");
    assert!(stdout.contains("2 tasks"), "missing footer: {stdout}");
}

#[test]
fn next_filter_multiple_set_args() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);

    common::execute_dawn(&db)
        .args(["1,2", "2,3"])
        .assert()
        .success()
        .stdout(contains("one").and(contains("two")).and(contains("three")));
}

#[test]
fn next_filter_nonexistent_index_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .arg("99,100")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

// ── Filter: index range (a-b) → next table ──

#[test]
fn next_filter_bare_range_returns_subset() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three", "four", "five"]);

    let stdout = common::run_stdout(common::execute_dawn(&db).arg("1-3"));

    // Index↔description mapping is not stable (see common::setup_tasks).
    // Range filter `1-3` selects exactly 3 of the 5 seeded tasks.
    let present = ["one", "two", "three", "four", "five"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 3, "expected 3 of 5 tasks to match: {stdout}");
    assert!(stdout.contains("3 tasks"), "missing footer: {stdout}");
}

#[test]
fn next_filter_descending_range_swaps_and_matches() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);

    common::execute_dawn(&db)
        .arg("3-1")
        .assert()
        .success()
        .stdout(
            contains("one")
                .and(contains("two"))
                .and(contains("three"))
                .and(contains("3 tasks")),
        );
}

// Equal-bounds range smoke check; collapse to single Index is unit-tested in filter.rs.
#[test]
fn next_filter_equal_bounds_range_matches_single_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["only"]);

    common::execute_dawn(&db)
        .arg("1-1")
        .assert()
        .success()
        .stdout(
            contains("only")
                .and(contains("1 task"))
                .and(contains("1 tasks").not())
                .and(contains("Last modified").not()),
        );
}

#[test]
fn next_filter_set_with_range_and_index() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three", "four"]);

    let stdout = common::run_stdout(common::execute_dawn(&db).arg("1-2,3"));

    let present = ["one", "two", "three", "four"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 3, "expected 3 of 4 tasks to match: {stdout}");
    assert!(stdout.contains("3 tasks"), "missing footer: {stdout}");
}

#[test]
fn next_filter_out_of_bounds_range_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "only"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .arg("99-100")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

// E2E for `tpr.row_id BETWEEN ? AND ?` AND-ed with the FTS MATCH clause.
#[test]
fn next_filter_range_combined_with_word_filter() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(
        &db,
        &["alpha foo", "bravo foo", "charlie foo", "delta", "echo"],
    );

    common::execute_dawn(&db)
        .args(["1-5", "foo"])
        .assert()
        .success()
        .stdout(
            contains("foo")
                .count(3)
                .and(contains("3 tasks"))
                .and(contains("delta").not())
                .and(contains("echo").not()),
        );
}

// ── Malformed sets demote whole token to a word ──

#[test]
fn next_filter_set_with_invalid_segment_demotes_to_word() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .arg("1,invalid")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn next_filter_set_with_zero_segment_demotes_to_word() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "one"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .arg("1,0")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}
