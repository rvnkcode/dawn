mod common;

#[test]
fn add_single_task_prints_counter_1() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
}

#[test]
fn add_two_tasks_counter_increments() {
    let (_dir, db) = common::test_db();
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
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db).args(["add", ""]).assert().code(2);
}

#[test]
fn add_whitespace_only_description_rejected() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db).args(["add", "   "]).assert().code(2);
}

#[test]
fn add_unquoted_multiword_joins_words() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy", "milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
    let output = common::dawn_cmd(&db)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(
        stdout.contains("buy milk"),
        "stdout missing joined description: {stdout}"
    );
}

#[test]
fn add_with_preceding_filter_joins_filter_and_words() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["1", "add", "buy", "milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
    let output = common::dawn_cmd(&db)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 stdout");
    assert!(
        stdout.contains("1 buy milk"),
        "stdout missing joined description: {stdout}"
    );
}

#[test]
fn add_missing_description_rejected() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db).args(["add"]).assert().code(2);
}
