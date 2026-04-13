mod common;

use tempfile::TempDir;

#[test]
fn next_with_no_tasks_prints_no_matches() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .assert()
        .success()
        .stdout("No matches.\n");
}

#[test]
fn next_with_one_task_prints_singular_footer() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
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
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
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

#[test]
fn next_lists_all_pending_tasks() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .args(["add", "alpha"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "bravo"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "charlie"])
        .assert()
        .success();
    let output = common::dawn_cmd(&db)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(stdout.contains("alpha"), "stdout missing 'alpha': {stdout}");
    assert!(stdout.contains("bravo"), "stdout missing 'bravo': {stdout}");
    assert!(
        stdout.contains("charlie"),
        "stdout missing 'charlie': {stdout}"
    );
    assert!(
        stdout.contains("3 tasks"),
        "stdout missing footer: {stdout}"
    );
}
