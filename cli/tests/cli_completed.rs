mod common;

use common::{assert_pty_exit, dawn_pty, delete_via_pty, select_option};
use predicates::{boolean::PredicateBooleanExt, str::contains};

// ── A. Empty / footer rendering ──

// dawn completed
// No matches.
#[test]
fn completed_with_no_tasks_prints_no_matches() {
    let (_dir, db) = common::test_db();

    common::execute_dawn(&db)
        .arg("completed")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No matches."));
}

// dawn add alpha
// dawn add beta
// dawn completed
// No matches.
#[test]
fn completed_with_only_pending_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["alpha", "beta"]);

    common::execute_dawn(&db)
        .arg("completed")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No matches."));
}

// dawn add "buy milk"
// dawn 1 done
// dawn "nonexistent" completed
// No matches.
#[test]
fn completed_filter_with_no_match_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::execute_dawn(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    common::execute_dawn(&db)
        .args(["1", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .args(["nonexistent", "completed"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("No matches."));
}

// dawn add apple
// dawn add banana
// dawn 1-2 done
// dawn completed
// - apple
// - banana
// 2 tasks
#[test]
fn completed_renders_multiple_tasks_with_plural_footer() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["apple", "banana"]);
    common::execute_dawn(&db)
        .args(["1-2", "done"])
        .assert()
        .success();

    common::execute_dawn(&db)
        .arg("completed")
        .assert()
        .success()
        .stdout(contains("apple"))
        .stdout(contains("banana"))
        .stdout(contains("2 tasks"));
}

// ── B. Status restriction ──

// dawn add "done-task"
// dawn add "pending-task"
// dawn add "deleted-task"
// dawn "done-" done
// dawn "delete-" delete
// dawn completed
// - done-task
// 1 task
#[test]
fn completed_excludes_pending_and_deleted_tasks() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["done-task", "pending-task", "deleted-task"]);
    common::execute_dawn(&db)
        .args(["done-", "done"])
        .assert()
        .success();
    delete_via_pty(&db, "deleted-");

    common::execute_dawn(&db)
        .arg("completed")
        .assert()
        .success()
        .stdout(contains("done-task"))
        .stdout(contains("1 task"))
        .stdout(contains("pending-task").not())
        .stdout(contains("deleted-task").not());
}

// ── C. Sort order ──

// dawn add alpha
// dawn add bravo
// dawn add charlie
// dawn 2 done
// dawn 1 done
// dawn 1 done
// dawn completed
// - bravo
// - alpha
// - charlie
#[test]
fn completed_sorted_by_completion_time_ascending() {
    let (_dir, db) = common::test_db();
    // Add tasks with 1s gaps so each task gets a unique entry second
    let add = |desc: &str| {
        common::execute_dawn(&db)
            .args(["add", desc])
            .assert()
            .success();
    };
    add("alpha");
    std::thread::sleep(std::time::Duration::from_secs(1));
    add("bravo");
    std::thread::sleep(std::time::Duration::from_secs(1));
    add("charlie");
    // Complete in non-creation order — bravo, alpha, charlie — with 1s gaps.
    let done = |index: &str| {
        common::execute_dawn(&db)
            .args([index, "done"])
            .assert()
            .success();
    };
    done("2");
    std::thread::sleep(std::time::Duration::from_secs(1));
    done("1");
    std::thread::sleep(std::time::Duration::from_secs(1));
    // Index changes after each done
    done("1");

    common::execute_dawn(&db)
        .arg("completed")
        .assert()
        .success()
        .stdout(predicates::str::is_match(r"(?s)bravo.*alpha.*charlie").unwrap());
}

// ── D. Filter matching ──

// dawn add "shared apple"
// dawn add "shared banana"
// dawn add "lonely cherry"
// dawn 1-3 done       (PTY: "All" on bulk confirm)
// dawn completed "shared"
// - shared apple
// - shared banana
// 2 tasks
#[test]
fn completed_filter_returns_matching_subset() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["shared apple", "shared banana", "lonely cherry"]);
    // 3 tasks ≥ BULK_CONFIRM_THRESHOLD; "All" approves remaining without per-task prompts
    let mut p = dawn_pty(&db, &["1-3", "done"]);
    p.exp_string("Complete task")
        .expect("first complete prompt");
    select_option(&mut p, "All");
    p.exp_string("Completed 3 tasks.").expect("done footer");
    assert_pty_exit(&mut p, 0);

    common::execute_dawn(&db)
        .args(["completed", "shared"])
        .assert()
        .success()
        .stdout(contains("shared apple"))
        .stdout(contains("shared banana"))
        .stdout(contains("lonely cherry").not())
        .stdout(contains("2 tasks"));
}
