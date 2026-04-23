mod common;

// TODO: `done` / `delete` 서브커맨드가 추가되면 info table 의 conditional 행
// 검증을 E2E 로 확장한다:
//   - 완료된 태스크: `End` 행 렌더 + Status 가 `Completed`
//   - 삭제된 태스크: `Deleted` 행 렌더 + Status 가 `Deleted`
//   - 완료 + 삭제가 모두 설정된 태스크: 두 행 모두 렌더
// 현재는 pending 상태만 CLI 로 도달 가능하므로 pending 경로만 검증한다.
// (conditional 행 렌더 자체는 `cli/src/table/info_table.rs` 유닛 테스트가 커버한다.)

#[test]
fn info_single_index_renders_all_base_rows() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let out = common::dawn_cmd(&db).arg("1").output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    for header in [
        "ID",
        "Description",
        "Status",
        "Entered",
        "Last modified",
        "UID",
    ] {
        assert!(stdout.contains(header), "missing row {header}: {stdout}");
    }
    assert!(stdout.contains("buy milk"), "missing description: {stdout}");
    assert!(stdout.contains("Pending"), "missing status: {stdout}");
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn info_omits_end_and_deleted_rows_for_pending() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let out = common::dawn_cmd(&db).arg("1").output().expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(!stdout.contains("End"), "unexpected End row: {stdout}");
    assert!(
        !stdout.contains("Deleted"),
        "unexpected Deleted row: {stdout}"
    );
}

#[test]
fn info_renders_uid_row_with_valid_uid() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();
    let first_out = common::dawn_cmd(&db).arg("1").output().expect("run");
    let first_stdout = String::from_utf8(first_out.stdout).expect("utf8 stdout");
    let uid = first_stdout
        .lines()
        .find(|line| line.trim_start().starts_with("UID"))
        .and_then(|line| line.split_whitespace().nth(1))
        .expect("UID value in info table")
        .to_string();
    assert_eq!(uid.len(), 12, "UID must be 12 chars: {uid}");

    let second_out = common::dawn_cmd(&db).arg(&uid).output().expect("run");
    assert!(second_out.status.success());
    let second_stdout = String::from_utf8(second_out.stdout).expect("utf8 stdout");
    assert!(
        second_stdout.contains(&uid),
        "info by UID missing UID row: {second_stdout}"
    );
    assert!(
        second_stdout.contains("buy milk"),
        "info by UID missing description: {second_stdout}"
    );
}

#[test]
fn info_multiple_bare_args_renders_each_task() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);
    let out = common::dawn_cmd(&db)
        .args(["1", "2"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
}

#[test]
fn info_nonexistent_index_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .arg("99")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn info_nonexistent_uid_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .arg("aaaaaaaaaaaa")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn info_empty_db_with_index_prints_no_matches() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .arg("1")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

#[test]
fn next_and_info_render_together_when_set_and_bare_given() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);
    let out = common::dawn_cmd(&db)
        .args(["1,2", "3"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");

    let next_footer = stdout.find("2 tasks").expect("next footer missing");
    let info_marker = stdout.find("Last modified").expect("info table missing");
    assert!(
        next_footer < info_marker,
        "next must render before info: {stdout}"
    );

    assert!(stdout.contains("one"), "next missing 'one': {stdout}");
    assert!(stdout.contains("two"), "next missing 'two': {stdout}");
    assert!(stdout.contains("three"), "info missing 'three': {stdout}");
}
