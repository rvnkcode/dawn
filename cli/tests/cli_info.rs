mod common;

use common::{delete_via_pty, extract_uuid, run_stdout};
use predicates::{
    prelude::PredicateBooleanExt,
    str::{contains, is_empty},
};

#[test]
fn info_single_index_renders_all_base_rows() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .arg("1")
        .assert()
        .success()
        .stdout(
            contains("ID")
                .and(contains("Description"))
                .and(contains("Status"))
                .and(contains("Entered"))
                .and(contains("Last modified"))
                .and(contains("UUID"))
                .and(contains("buy milk"))
                .and(contains("Pending"))
                .and(contains("End").not())
                .and(contains("Deleted").not()),
        )
        .stderr(is_empty());
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

    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(
            contains("End")
                .and(contains("Completed"))
                .and(contains("Deleted").not()),
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

    // "Deleted" appears twice: once as Status value, once as row label.
    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(
            contains("Deleted")
                .count(2)
                .and(contains("End").not())
                .and(contains("Completed").not()),
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

    // Terminal status is "Deleted"; row label "Deleted" also rendered (count 2).
    common::execute_dawn(&db)
        .arg(&uuid)
        .assert()
        .success()
        .stdout(
            contains("End")
                .and(contains("Deleted").count(2))
                .and(contains("Completed").not()),
        );
}

#[test]
fn info_multiple_bare_args_renders_each_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1", "2"])
        .assert()
        .success()
        .stdout(contains("one").and(contains("two")));
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

    common::execute_dawn(&db)
        .args(["1,2", "3"])
        .assert()
        .success()
        .stdout(
            contains("one")
                .and(contains("two"))
                .and(contains("three"))
                .and(contains("Last modified").count(3))
                .and(contains("3 tasks").not())
                .and(contains("2 tasks").not()),
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

    common::execute_dawn(&db)
        .arg("investigate")
        .assert()
        .success()
        .stdout(
            contains("investigate flaky build")
                .and(contains("buy milk").not())
                .and(contains("1 task")),
        );
}

#[test]
fn bare_index_with_range_routes_to_info_with_merged_filter() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);

    common::execute_dawn(&db)
        .args(["1", "2-3"])
        .assert()
        .success()
        .stdout(
            contains("one")
                .and(contains("two"))
                .and(contains("three"))
                .and(contains("Last modified").count(3))
                .and(contains("3 tasks").not())
                .and(contains("2 tasks").not()),
        );
}

#[test]
fn bare_with_nonexistent_id_and_set_filter_exits_cleanly() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);

    common::execute_dawn(&db)
        .args(["1,2", "99"])
        .assert()
        .success()
        .stdout(
            contains("one")
                .and(contains("two"))
                .and(contains("Last modified").count(2)),
        );
}
