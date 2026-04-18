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

// ── Filter: set (comma-separated) → next table ──

fn setup_three_tasks(db: &std::path::Path) {
    let descs = ["one", "two", "three"];
    for (i, desc) in descs.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        common::dawn_cmd(db).args(["add", desc]).assert().success();
    }
}

#[test]
fn next_filter_set_two_indices() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    setup_three_tasks(&db);
    let out = common::dawn_cmd(&db).arg("1,2").output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(out.status.success());
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert!(!stdout.contains("three"), "unexpected 'three': {stdout}");
}

#[test]
fn next_filter_multiple_set_args() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    setup_three_tasks(&db);
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
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    setup_three_tasks(&db);
    common::dawn_cmd(&db)
        .arg("99,100")
        .assert()
        .success()
        .stdout("No matches.\n");
}

// ── Silent invalid (set with partial invalid) ──

#[test]
fn next_filter_set_with_invalid_silently_drops() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    setup_three_tasks(&db);
    let out = common::dawn_cmd(&db)
        .arg("1,invalid")
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8(out.stderr).expect("utf8");
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn next_filter_set_with_zero_silently_drops() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    setup_three_tasks(&db);
    let out = common::dawn_cmd(&db).arg("1,0").output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8(out.stderr).expect("utf8");
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

// ── All-invalid → "No matches." exit 0 ──

#[test]
fn all_invalid_single_prints_no_matches() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .arg("invalid")
        .assert()
        .success()
        .stdout("No matches.\n");
}

#[test]
fn all_invalid_set_prints_no_matches() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .arg("invalid,xyz")
        .assert()
        .success()
        .stdout("No matches.\n");
}

#[test]
fn zero_bare_prints_no_matches() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("dawn.db");
    common::dawn_cmd(&db)
        .arg("0")
        .assert()
        .success()
        .stdout("No matches.\n");
}
