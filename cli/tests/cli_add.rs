mod common;

use tempfile::TempDir;

#[test]
fn add_single_task_prints_counter_1() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
}

#[test]
fn add_two_tasks_counter_increments() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .args(["add", "first"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
    common::dawn_cmd(&db)
        .args(["add", "second"])
        .assert()
        .success()
        .stdout("Created task 2.\n");
}

#[test]
fn add_empty_description_rejected() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db).args(["add", ""]).assert().code(2);
}

#[test]
fn add_whitespace_only_description_rejected() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db).args(["add", "   "]).assert().code(2);
}
