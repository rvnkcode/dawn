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
        .arg("abc1efghijkl")
        .assert()
        .code(1)
        .stderr("No matches.\n");
}

// Regression: nanoid SAFE alphabet allows UIDs starting with '-'. Without
// `allow_hyphen_values` on the pre-filter, clap rejects them as unknown flags.
#[test]
fn info_hyphen_prefixed_uid_does_not_panic_clap() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "only"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .arg("-Abc1efghijk")
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

// ── Taskwarrior-style dispatch: any bare id/uuid → info with merged filter ──

#[test]
fn mixed_set_and_bare_resolves_to_info_with_merged_ids() {
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two", "three"]);
    let out = common::dawn_cmd(&db)
        .args(["1,2", "3"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");

    // Taskwarrior rule: presence of bare "3" dispatches to info.
    // All three ids are merged via UNION, so every task renders as an info table.
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert!(stdout.contains("three"), "missing 'three': {stdout}");

    // Info renders each task's "Last modified" row; three tasks → three occurrences.
    let last_modified_count = stdout.matches("Last modified").count();
    assert_eq!(
        last_modified_count, 3,
        "expected 3 info tables, got {last_modified_count}: {stdout}"
    );

    // No next footer should appear — info path is status-agnostic and does not
    // print the "N tasks" summary.
    assert!(
        !stdout.contains("3 tasks") && !stdout.contains("2 tasks"),
        "unexpected next footer: {stdout}"
    );
}

#[test]
fn non_id_bare_routes_to_next_with_word_filter() {
    let (_dir, db) = common::test_db();
    common::dawn_cmd(&db)
        .args(["add", "investigate flaky build"])
        .assert()
        .success();
    common::dawn_cmd(&db)
        .args(["add", "buy milk"])
        .assert()
        .success();

    // "investigate" parses as neither Index nor UniqueID, so has_bare_id stays
    // false and the command resolves to `next` with a words filter rather than
    // `info`. Only the matching task renders.
    let out = common::dawn_cmd(&db)
        .arg("investigate")
        .output()
        .expect("run");
    assert!(out.status.success(), "expected next path to succeed");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("investigate flaky build"),
        "next table missing matching task: {stdout}"
    );
    assert!(
        !stdout.contains("buy milk"),
        "next table should not include non-matching task: {stdout}"
    );
    assert!(stdout.contains("1 task"), "missing next footer: {stdout}");
}

#[test]
fn bare_with_nonexistent_id_and_set_filter_exits_cleanly() {
    // Regression: previously, `dawn 1,2 99` triggered both next (1,2) and info (99),
    // producing a partial-failure state (stdout had next output + stderr had
    // "No matches." + exit 1). After the refactor, this resolves to a single info
    // call with merged filter {1, 2, 99}. Tasks 1 and 2 exist, so info succeeds.
    let (_dir, db) = common::test_db();
    common::setup_tasks(&db, &["one", "two"]);
    let out = common::dawn_cmd(&db)
        .args(["1,2", "99"])
        .output()
        .expect("run");
    assert!(out.status.success(), "expected success via merged info");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.contains("one"), "missing 'one': {stdout}");
    assert!(stdout.contains("two"), "missing 'two': {stdout}");
    assert_eq!(stdout.matches("Last modified").count(), 2);
}
