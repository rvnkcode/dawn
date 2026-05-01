mod common;

#[test]
fn next_with_no_tasks_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn next_with_one_task_prints_singular_footer() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let output = common::dawn_cmd(&db)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(
        stdout.contains("buy milk"),
        "stdout missing description: {stdout}"
    );
    assert!(
        stdout.contains("1 task"),
        "stdout missing singular footer: {stdout}"
    );
    assert!(
        !stdout.contains("1 tasks"),
        "stdout has plural form: {stdout}"
    );
}

#[test]
fn next_with_multiple_tasks_prints_plural_footer() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "two"])
        .assert()
        .success();
    let output = common::dawn_cmd(&db)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(stdout.contains("one"), "stdout missing 'one': {stdout}");
    assert!(stdout.contains("two"), "stdout missing 'two': {stdout}");
    assert!(
        stdout.contains("2 tasks"),
        "stdout missing plural footer: {stdout}"
    );
}

// ── Filter: set (comma-separated) → next table ──

#[test]
fn next_filter_set_two_indices() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);
    let out = common::dawn_cmd(&db).arg("1,2").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
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
    let out = common::dawn_cmd(&db)
        .args(["1,2", "2,3"])
        .output()
        .expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
    assert!(stdout.contains("one"));
    assert!(stdout.contains("two"));
    assert!(stdout.contains("three"));
}

#[test]
fn next_filter_nonexistent_index_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();
    common::dawn_cmd(&db)
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
    let out = common::dawn_cmd(&db).arg("1-3").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
    // Index↔description mapping is not stable (see common::setup_tasks).
    // Range filter `1-3` selects exactly 3 of the 5 seeded tasks.
    let present = ["one", "two", "three", "four", "five"]
        .iter()
        .filter(|d| stdout.contains(*d))
        .count();
    assert_eq!(present, 3, "expected 3 of 5 tasks to match: {stdout}");
    assert!(stdout.contains("3 tasks"), "missing footer: {stdout}");
}

// `3-1` auto-swaps to `1-3` in IndexRange::new — proves the swap survives the
// parser → Filter → query_builder BETWEEN clause path.
#[test]
fn next_filter_descending_range_swaps_and_matches() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);
    let out = common::dawn_cmd(&db).arg("3-1").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
    assert!(stdout.contains("one"));
    assert!(stdout.contains("two"));
    assert!(stdout.contains("three"));
    assert!(stdout.contains("3 tasks"), "missing footer: {stdout}");
}

// Equal-bounds range smoke check; collapse to single Index is unit-tested in filter.rs.
#[test]
fn next_filter_equal_bounds_range_matches_single_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["only"]);
    let out = common::dawn_cmd(&db).arg("1-1").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
    assert!(stdout.contains("only"), "missing task: {stdout}");
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
fn next_filter_set_with_range_and_index() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three", "four"]);
    let out = common::dawn_cmd(&db).arg("1-2,3").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
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
    common::dawn_cmd(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .arg("99-100")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

// `1-1` collapses to Index(1) at the parser, but the range branch does not
// set has_bare_id, so it routes to `next` rather than `info` — unlike bare
// `dawn 1`, which produces the same Index(1) filter yet dispatches to info.
// Distinguishing signal: next prints the "N tasks" footer and never the
// vertical Info table header "Last modified".
#[test]
fn bare_equal_bounds_range_routes_to_next_not_info() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);
    let out = common::dawn_cmd(&db).arg("1-1").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
    assert!(
        stdout.contains("1 task"),
        "missing singular next footer: {stdout}"
    );
    assert!(
        !stdout.contains("1 tasks"),
        "unexpected plural footer: {stdout}"
    );
    assert_eq!(
        stdout.matches("Last modified").count(),
        0,
        "unexpected Info table render: {stdout}"
    );
}

// E2E for `tpr.row_id BETWEEN ? AND ?` AND-ed with the FTS MATCH clause.
#[test]
fn next_filter_range_combined_with_word_filter() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(
        &db,
        &["alpha foo", "bravo foo", "charlie foo", "delta", "echo"],
    );
    let stdout = common::run_stdout(common::dawn_cmd(&db).args(["1-5", "foo"]));
    assert_eq!(
        stdout.matches("foo").count(),
        3,
        "expected 3 foo matches: {stdout}"
    );
    assert!(stdout.contains("3 tasks"), "missing footer: {stdout}");
    assert!(!stdout.contains("delta"), "delta leaked through: {stdout}");
    assert!(!stdout.contains("echo"), "echo leaked through: {stdout}");
}

// ── Malformed sets demote whole token to a word ──

#[test]
fn next_filter_set_with_invalid_segment_demotes_to_word() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();
    // "1,invalid" fails the strict set syntax, falls through, and is searched
    // for verbatim in descriptions. No task contains that string.
    common::dawn_cmd(&db)
        .arg("1,invalid")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn next_filter_set_with_zero_segment_demotes_to_word() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "one"])
        .assert()
        .success();
    // "0" is rejected by the index segment alternation, so "1,0" is treated as
    // a single word, not a partial index set.
    common::dawn_cmd(&db)
        .arg("1,0")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

// ── All-invalid → "No matches." exit 1 ──

#[test]
fn all_invalid_single_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .arg("invalid")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn zero_bare_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .arg("0")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}
