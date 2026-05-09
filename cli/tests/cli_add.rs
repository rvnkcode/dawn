mod common;

use predicates::str::{contains, is_empty};

/*
 * dawn add "buy milk"
 * Created task 1.
 */
#[test]
fn add_single_task_prints_counter_1() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
}

/*
 * dawn add "first"
 * Created task 1.
 * dawn add "second"
 * Created task 2.
 */
#[test]
fn add_two_tasks_counter_increments() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "first"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
    common::execute_dawn(&db)
        .args(["add", "second"])
        .assert()
        .success()
        .stdout("Created task 2.\n");
}

/*
 * dawn add ""
 * Additional text must be provided.
 */
#[test]
fn add_empty_description_rejected() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", ""])
        .assert()
        .code(2)
        .stdout(is_empty())
        .stderr("Additional text must be provided.\n");
}

/*
 * dawn add
 * Additional text must be provided.
 */
#[test]
fn add_missing_description_rejected() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add"])
        .assert()
        .code(2)
        .stdout(is_empty())
        .stderr("Additional text must be provided.\n");
}

/*
 * dawn add "   "
 * Additional text must be provided.
 */
#[test]
fn add_whitespace_only_description_rejected() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "   "])
        .assert()
        .code(2)
        .stdout(is_empty())
        .stderr("Additional text must be provided.\n");
}

/*
 * dawn add buy milk
 * Created task 1.
 * dawn
 * 1 (age) buy milk
 */
#[test]
fn add_unquoted_multiple_words_joins_words() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy", "milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("buy milk"));
}

/*
 * dawn 1 add buy milk
 * Created task 1.
 * dawn
 * 1 (age) 1 buy milk
 */
#[test]
fn add_with_preceding_filter_joins_filter_and_words() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["1", "add", "buy", "milk"])
        .assert()
        .success()
        .stdout("Created task 1.\n");
    common::execute_dawn(&db)
        .assert()
        .success()
        .stdout(contains("1 buy milk"));
}
