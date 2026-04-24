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
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert!(!stdout.contains("three"), "unexpected 'three': {stdout}");
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
fn all_invalid_set_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .arg("invalid,xyz")
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
