---
title: Test Cases
---

## Domain Layer (`lib/src/domain`)

### `task/unique_id.rs`

| Test | 설명 |
| --- | --- |
| `unique_id_new` | 새로 생성한 UniqueID가 유효한 12자리 문자열인지 확인한다. |
| `from_str_valid` | 유효한 12자리 문자열을 UniqueID로 파싱할 수 있다. |
| `from_str_short` | 길이가 부족한 문자열(e.g. 11자리)은 파싱에 실패한다. |
| `from_str_long` | 길이가 초과한 문자열(e.g. 13자리)은 파싱에 실패한다. |
| `from_str_with_space` | 중간에 공백이 포함되면 파싱에 실패한다. |
| `from_str_with_disallowed_character` | 허용되지 않은 문자(e.g. `!`)가 포함되면 파싱에 실패한다. |
| `from_str_whitespace` | 앞뒤 공백이 있는 문자열은 트림되어 정상 파싱된다. |
| `from_str_whitespace_only` | 공백으로만 이루어진 문자열은 파싱에 실패한다. |

### `task/index.rs`

| Test | 설명 |
| --- | --- |
| `index_new_valid` | 양의 정수로 Index를 정상 생성한다. |
| `index_new_zero` | 0은 유효한 Index가 아니므로 생성에 실패한다. |
| `from_str_valid` | 숫자 문자열을 Index로 파싱할 수 있다. |
| `from_str_zero` | "0" 문자열은 파싱에 실패한다. |
| `from_str_non_numeric` | 숫자가 아닌 문자열은 파싱에 실패한다. |
| `from_str_empty` | 빈 문자열은 파싱에 실패한다. |
| `from_str_whitespace` | 앞뒤 공백이 있는 문자열은 트림되어 정상 파싱된다. |
| `from_str_overflow` | u32 범위를 초과하는 값은 오버플로 에러를 반환한다. |

### `task/description.rs`

| Test | 설명 |
| --- | --- |
| `description_new_valid` | 유효한 문자열로 Description을 생성한다. |
| `description_new_empty_string` | 빈 문자열은 Description이 될 수 없다. |
| `description_new_whitespace_only` | 공백만 있는 문자열은 Description이 될 수 없다. |
| `description_new_trims_whitespace` | 생성 시 앞뒤 공백이 트림된다. |
| `from_str_valid` | 문자열로부터 Description을 파싱할 수 있다. |

### `task/timestamp.rs`

| Test | 설명 |
| --- | --- |
| `timestamp_new_valid` | 유효한 Unix 시각 값으로 Timestamp를 생성할 수 있다. |
| `timestamp_new_invalid` | 유효하지 않은 시각 값으로는 Timestamp를 생성할 수 없다. |

### `task/filter.rs`

| Test | 설명 |
| --- | --- |
| `with_uids_single` | 단일 UniqueID 를 포함한 필터를 생성한다. |
| `with_uids_multiple` | 여러 UniqueID 를 포함한 필터를 생성한다. |
| `with_uids_deduplicates` | 중복 UniqueID 는 자동으로 제거된다. |
| `with_indices_single` | 단일 Index 를 포함한 필터를 생성한다. |
| `with_indices_multiple` | 여러 Index 를 포함한 필터를 생성한다. |
| `with_indices_deduplicates` | 중복 Index 는 자동으로 제거된다. |
| `with_statuses_single` | 단일 Status 를 포함한 필터를 생성한다. |
| `with_statuses_multiple` | 여러 Status 를 포함한 필터를 생성한다. |
| `with_statuses_deduplicates` | 중복 Status 는 자동으로 제거된다. |
| `is_empty_with_uids_only` | UID 만 있어도 필터는 비어있지 않다. |
| `is_empty_with_indices_only` | Index 만 있어도 필터는 비어있지 않다. |
| `is_empty_with_statuses_only` | Status 만 있어도 필터는 비어있지 않다. |
| `is_empty_with_all` | 모든 항목이 있는 필터는 비어있지 않다. |
| `with_uids_extends` | `with_uids` 를 두 번 호출하면 두 UID 가 모두 누적되어 길이 2 의 set 이 된다. |
| `with_indices_extends` | `with_indices` 를 두 번 호출하면 두 Index 가 모두 누적되어 길이 2 의 set 이 된다. |
| `with_statuses_extends` | `with_statuses` 를 두 번 호출하면 두 Status 가 모두 누적되어 길이 2 의 set 이 된다. |

### `task.rs`

| Test | 설명 |
| --- | --- |
| `status_is_pending_when_not_completed_or_deleted` | completed 와 deleted 가 모두 None 이면 status() 는 Status::Pending 을 반환하고 문자열 표기는 "Pending" 이다. |
| `status_is_completed_when_completed_is_some` | completed 만 설정되면 status() 는 Status::Completed 를 반환하고 문자열 표기는 "Completed" 이다. |
| `status_is_deleted_when_deleted_is_some` | deleted 만 설정되면 status() 는 Status::Deleted 를 반환하고 문자열 표기는 "Deleted" 이다. |
| `status_is_deleted_when_both_completed_and_deleted` | completed 와 deleted 가 모두 설정되면 deleted 가 우선되어 status() 는 Status::Deleted 를 반환한다. |
| `task_modification_is_empty_when_all_none` | 모든 필드가 None 이면 수정 내역이 비어있다고 판정한다. |
| `task_modification_is_not_empty_with_description` | description 변경이 있으면 수정 내역이 존재한다. |
| `task_modification_is_not_empty_with_completed` | completed 설정이 있으면 수정 내역이 존재한다. |
| `task_modification_is_not_empty_with_completed_cleared` | completed 해제 설정이 있으면 수정 내역이 존재한다. |
| `task_modification_is_not_empty_with_deleted` | deleted 설정이 있으면 수정 내역이 존재한다. |
| `task_modification_is_not_empty_with_deleted_cleared` | deleted 해제 설정이 있으면 수정 내역이 존재한다. |

## Outbound Layer (`lib/src/outbound`)

### `query_builder/tests.rs`

| Test | 설명 |
| --- | --- |
| `build_where_clause_with_empty_filter` | 빈 필터는 WHERE 절 없이 쿼리를 생성한다. |
| `build_where_clause_with_pending_only` | pending 상태만 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_completed_only` | completed 상태만 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_deleted_only` | deleted 상태만 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_two_statuses` | 두 개의 상태를 OR 로 결합한 WHERE 절을 생성한다. |
| `build_where_clause_with_all_statuses` | 세 상태 모두 지정하면 상태 조건이 생략된다. |
| `repeat_vars_single` | 단일 바인딩 변수 placeholder 를 생성한다. |
| `repeat_vars_multiple` | 여러 바인딩 변수 placeholder 를 콤마로 연결하여 생성한다. |
| `build_where_clause_with_single_uid` | 단일 UID 로 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_multiple_uids` | 여러 UID 로 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_single_index` | 단일 Index 로 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_multiple_indices` | 여러 Index 로 필터링하는 WHERE 절을 생성한다. |
| `build_where_clause_with_uid_and_index` | UID 와 Index 를 함께 사용하는 WHERE 절을 생성한다. |
| `build_where_clause_with_uid_and_status` | UID 와 Status 를 함께 사용하는 WHERE 절을 생성한다. |
| `build_where_clause_with_index_and_status` | Index 와 Status 를 함께 사용하는 WHERE 절을 생성한다. |
| `build_where_clause_with_uid_and_index_and_status` | UID, Index, Status 모두 사용하는 WHERE 절을 생성한다. |
| `build_update_clause_with_empty_modification` | 비어있는 수정 내역에 대해 에러를 반환한다. |
| `build_update_clause_with_empty_targets` | 대상이 없으면 에러를 반환한다. |
| `build_update_clause_with_description` | description 만 변경하는 UPDATE 절을 생성한다. |
| `build_update_clause_with_completed_set` | completed 를 설정하는 UPDATE 절을 생성한다. |
| `build_update_clause_with_completed_cleared` | completed 를 해제하는 UPDATE 절을 생성한다. |
| `build_update_clause_with_deleted_set` | deleted 를 설정하는 UPDATE 절을 생성한다. |
| `build_update_clause_with_deleted_cleared` | deleted 를 해제하는 UPDATE 절을 생성한다. |
| `build_update_clause_with_multiple_fields` | 여러 필드를 한 번에 변경하는 UPDATE 절을 생성한다. |
| `build_update_clause_with_multiple_targets` | 여러 대상을 한 번에 변경하는 UPDATE 절을 생성한다. |

### `sqlite/tests/init.rs`

| Test | 설명 |
| --- | --- |
| `create_schema_on_fresh_database` | 신규 DB 파일에 스키마를 정상적으로 생성한다. |
| `skip_migration_when_version_matches` | 버전이 일치하면 마이그레이션을 건너뛴다. |
| `error_when_database_version_is_newer` | DB 버전이 앱보다 최신이면 에러를 반환한다. |

### `sqlite/tests/crud.rs`

| Test | 설명 |
| --- | --- |
| `create_task_inserts_row` | 태스크 생성 시 행이 삽입된다. |
| `create_task_duplicate_id_returns_error` | 동일한 ID 로 중복 생성하면 에러를 반환한다. |
| `count_pending_returns_error_when_table_missing` | 테이블이 없으면 pending 수 조회가 에러를 반환한다. |
| `count_pending_returns_zero_when_no_tasks` | 태스크가 없으면 pending 수는 0 이다. |
| `count_pending_returns_count_of_pending_tasks` | pending 태스크의 수를 정확히 반환한다. |
| `count_pending_excludes_deleted_and_completed_tasks` | pending 수에서 completed 와 deleted 는 제외된다. |
| `update_tasks_changes_description` | 태스크의 description 을 수정한다. |
| `update_tasks_sets_completed` | 태스크를 완료 상태로 변경한다. |
| `update_tasks_clears_completed` | 태스크의 완료 상태를 해제한다. |
| `update_tasks_sets_deleted` | 태스크를 삭제 상태로 변경한다. |
| `update_tasks_clears_deleted` | 태스크의 삭제 상태를 해제한다. |
| `update_tasks_fires_modified_trigger` | 태스크 수정 시 modified 트리거가 발동한다. |
| `delete_single_task` | 단일 태스크를 삭제한다. |
| `delete_multiple_tasks` | 여러 태스크를 한 번에 삭제한다. |
| `delete_empty_targets_returns_error` | 삭제 대상이 비어있으면 에러를 반환한다. |

### `sqlite/tests/list.rs`

| Test | 설명 |
| --- | --- |
| `list_tasks_returns_empty_vec_when_no_tasks` | 태스크가 없으면 빈 Vec 을 반환한다. |
| `list_tasks_orders_by_entry_then_id` | entry 시간, 그 다음 ID 순으로 정렬해서 반환한다. |
| `list_tasks_filter_pending_only` | pending 태스크만 필터링해서 반환한다. |
| `list_tasks_filter_completed_only` | completed 태스크만 필터링해서 반환한다. |
| `list_tasks_filter_deleted_only` | deleted 태스크만 필터링해서 반환한다. |
| `list_tasks_no_filter_returns_all` | 필터가 없으면 모든 상태의 태스크를 반환한다. |
| `list_tasks_filter_two_statuses` | 두 가지 상태의 태스크를 함께 반환한다. |
| `list_tasks_filter_all_statuses` | 세 가지 상태 모두 지정해도 정상 동작한다. |
| `list_tasks_filter_single_uid` | 단일 UID 로 태스크를 조회한다. |
| `list_tasks_filter_multiple_uids` | 여러 UID 로 태스크를 조회한다. |
| `list_tasks_filter_nonexistent_uid` | 존재하지 않는 UID 는 빈 결과를 반환한다. |
| `list_tasks_filter_uid_with_status` | UID 와 Status 를 함께 필터링한다. |
| `list_tasks_filter_single_index` | 단일 Index 로 태스크를 조회한다. |
| `list_tasks_filter_multiple_indices` | 여러 Index 로 태스크를 조회한다. |
| `list_tasks_filter_index_with_completed_returns_empty` | Index 로 completed 를 조회하면 빈 결과다. |
| `list_tasks_filter_index_with_deleted_returns_empty` | Index 로 deleted 를 조회하면 빈 결과다. |
| `list_tasks_filter_nonexistent_index` | 존재하지 않는 Index 는 빈 결과를 반환한다. |
| `list_tasks_filter_uid_and_index` | UID 또는 Index 중 하나라도 매칭되는 태스크를 반환한다 (OR 결합). |

### `sqlite/tests/triggers.rs`

| Test | 설명 |
| --- | --- |
| `update_modified_when_description_changes` | description 변경 시 modified 값이 갱신된다. |
| `update_modified_when_completed_changes` | completed 변경 시 modified 값이 갱신된다. |
| `update_modified_when_deleted_changes` | deleted 변경 시 modified 값이 갱신된다. |
| `update_modified_when_entry_changes` | entry 변경 시 modified 값이 갱신된다. |
| `not_update_modified_when_same_value` | 동일한 값으로 UPDATE 하면 modified 는 갱신되지 않는다. |
| `assign_sequential_row_ids_to_pending_tasks` | pending 태스크에 순차 row ID 를 부여한다. |
| `exclude_deleted_and_completed_tasks_from_row_ids` | deleted 와 completed 는 row ID 부여 대상에서 제외된다. |
| `sync_insert_to_fts` | INSERT 시 FTS 테이블에 자동 동기화된다. |
| `sync_update_to_fts` | UPDATE 시 FTS 테이블에 자동 동기화된다. |
| `sync_delete_to_fts` | DELETE 시 FTS 테이블에 자동 동기화된다. |

## CLI Layer (`cli/src`)

### `filter.rs`

| Test | 설명 |
| --- | --- |
| `parses_single_uid_as_bare` | 단일 UID 인자를 bare 로 파싱한다. |
| `parses_single_index_as_bare` | 단일 Index 인자를 bare 로 파싱한다. |
| `parses_12_digit_numeric_as_uid` | 12자리 숫자 문자열은 UID 로 인식한다. |
| `parses_comma_separated_indices_as_set` | 콤마로 연결된 Index 들은 set 으로 파싱한다. |
| `parses_mixed_index_and_uid_as_set` | Index 와 UID 가 혼합된 set 을 정상 파싱한다. |
| `silently_drops_single_invalid_bare` | bare 자리의 유효하지 않은 인자는 조용히 버린다. |
| `silently_drops_zero_bare` | bare 자리에 있는 "0" 은 조용히 버린다. |
| `silently_drops_invalid_segment_in_set` | set 내의 잘못된 세그먼트는 조용히 버린다. |
| `silently_drops_all_invalid_set` | set 전체가 잘못되어 있으면 모두 버린다. |
| `silently_drops_non_ascii` | 비 ASCII 문자열은 조용히 버린다. |
| `empty_string_yields_empty` | 빈 문자열은 빈 필터를 반환한다. |
| `double_comma_rejected_as_malformed` | 연속된 콤마("1,,2")는 잘못된 형식으로 처리한다. |
| `trailing_comma_rejected_as_malformed` | 끝에 콤마("1,")가 있으면 잘못된 형식으로 처리한다. |
| `leading_comma_rejected_as_malformed` | 앞에 콤마(",1")가 있으면 잘못된 형식으로 처리한다. |
| `outer_whitespace_trimmed` | 양 끝 공백은 트림하여 파싱한다. |
| `whitespace_around_comma_rejected` | 콤마 주변 공백은 잘못된 형식으로 처리한다. |
| `multiple_bare_args_merge_into_bare` | 여러 bare 인자는 bare 로 병합된다. |
| `multiple_set_args_merge_into_set` | 여러 set 인자는 set 으로 병합된다. |
| `bare_and_set_kept_separate` | bare 와 set 은 서로 다른 버킷으로 구분된다. |
| `duplicates_within_set_are_deduped` | set 내 중복 값은 제거된다. |
| `into_filters_builds_set_filter_from_set_terms` | set 항목은 첫 번째 Filter 에, bare 는 비어있는 두 번째 Filter 로 분리되어 반환된다. |
| `into_filters_builds_bare_filter_from_bare_terms` | bare 항목은 두 번째 Filter 에, set 은 비어있는 첫 번째 Filter 로 분리되어 반환된다. |
| `into_filters_splits_set_and_bare_terms_independently` | set 과 bare 가 혼합되어 있으면 각각 독립된 Filter 로 반환된다. |
| `into_filters_yields_two_empty_filters_for_no_valid_terms` | 입력이 비어있으면 두 Filter 모두 빈 상태로 반환된다. |
| `into_filters_yields_two_empty_filters_for_all_invalid_terms` | 입력이 전부 잘못된 항목이면 두 Filter 모두 빈 상태로 반환된다. |

### `table/base_table.rs`

| Test | 설명 |
| --- | --- |
| `render_includes_headers_and_row_data` | 렌더링 결과에 헤더와 데이터 행이 포함된다. |

### `table/age.rs`

| Test | 설명 |
| --- | --- |
| `seconds` | 초 단위 경과 시간을 포맷한다. |
| `zero_delta` | 경과 시간이 0이면 "0s" 로 표기한다. |
| `minutes` | 분 단위 경과 시간을 포맷한다. |
| `hours` | 시간 단위 경과 시간을 포맷한다. |
| `days` | 일 단위 경과 시간을 포맷한다. |
| `days_just_under_two_weeks` | 2주 직전은 여전히 일 단위로 표기된다. |
| `weeks_at_two_week_boundary` | 2주 경계에서는 주 단위로 전환된다. |
| `weeks_eleven` | 11주는 주 단위로 표기된다. |
| `calendar_months_exact_three` | 정확히 3개월은 월 단위로 표기된다. |
| `calendar_months_borrow_still_weeks` | 월 경계를 넘지 않으면 주 단위로 유지된다. |
| `calendar_months_nine_across_year` | 연말/연초를 가로지르는 9개월을 정상 계산한다. |
| `year_and_months` | 1년 이상 + 개월은 함께 포맷된다. |
| `year_exact_no_months` | 정확히 1년은 월 없이 "1y" 로 표기된다. |
| `year_just_before_anniversary_shows_months` | 1주년 직전은 여전히 개월로 표기된다. |
| `two_years_no_months` | 정확히 2년은 월 없이 "2y" 로 표기된다. |
| `invalid_negative_delta` | 음수 경과 시간은 유효하지 않은 입력으로 처리한다. |

### `table/next_row.rs`

| Test | 설명 |
| --- | --- |
| `new_succeeds_with_valid_task` | 유효한 Task 로 생성한 NextRow 의 필드가 `["1", "30s", "buy milk"]` 로 렌더링된다. |
| `new_returns_missing_index_when_index_is_none` | Task 의 Index 가 None 이면 `NextRowError::MissingIndex` 를 반환한다. |

### `table/date_format.rs`

| Test | 설명 |
| --- | --- |
| `format_absolute_in_renders_local_date` | UTC 자정 타임스탬프를 KST 로 변환하여 `"2024-01-15 09:00:00"` 을 반환한다. |
| `format_absolute_in_crosses_date_boundary` | UTC 23:30 타임스탬프를 KST 로 변환하면 날짜 경계를 넘어 `"2024-01-15 08:30:00"` 을 반환한다. |
| `format_with_age_appends_parenthesized_age` | 절대 시각 뒤에 `" (30s)"` 형식의 경과 시간이 괄호로 덧붙는다. |

### `table/info_table.rs`

| Test | 설명 |
| --- | --- |
| `render_includes_base_rows_for_pending_task` | pending Task 의 렌더링에 ID/Description/Status/Entered/Last modified/UID 행과 "Pending" 상태, description, UID 가 포함되고 End/Deleted 는 포함되지 않는다. |
| `render_uses_dash_for_id_when_index_is_none` | Task 의 Index 가 None 이면 ID 행의 값이 `"-"` 로 렌더링된다. |
| `render_includes_end_row_when_completed_is_set` | completed 가 설정되어 있으면 렌더링에 "End" 행이 추가되고 "Deleted" 는 포함되지 않는다. |
| `render_includes_deleted_row_when_deleted_is_set` | deleted 가 설정되어 있으면 렌더링에 "Deleted" 행이 추가되고 "End" 는 포함되지 않는다. |
| `render_includes_both_end_and_deleted_rows_when_set` | completed 와 deleted 가 모두 설정되어 있으면 렌더링에 "End" 와 "Deleted" 행이 모두 포함된다. |

## E2E Tests (`cli/tests`)

### `cli_add.rs`

| Test | 설명 |
| --- | --- |
| `add_single_task_prints_counter_1` | 첫 태스크 추가 시 카운터 1 을 출력한다. |
| `add_two_tasks_counter_increments` | 태스크가 추가될수록 카운터가 증가한다. |
| `add_empty_description_rejected` | 빈 description 으로는 태스크를 추가할 수 없다. |
| `add_whitespace_only_description_rejected` | 공백만 있는 description 은 거부된다. |
| `add_unquoted_multiword_joins_words` | 따옴표 없는 여러 단어는 공백으로 연결되어 저장된다. |
| `add_with_preceding_filter_joins_filter_and_words` | 앞쪽 필터 인자도 description 의 일부로 합쳐진다. |
| `add_missing_description_rejected` | description 이 없으면 태스크 추가가 거부된다. |

### `cli_next.rs`

| Test | 설명 |
| --- | --- |
| `next_with_no_tasks_prints_no_matches` | 태스크가 없으면 "No matches" 메시지를 출력한다. |
| `next_with_one_task_prints_singular_footer` | 태스크가 한 개면 단수형 footer 를 출력한다. |
| `next_with_multiple_tasks_prints_plural_footer` | 태스크가 여러 개면 복수형 footer 를 출력한다. |
| `next_filter_set_two_indices` | 두 개의 Index 로 구성된 set 필터를 적용한다. |
| `next_filter_multiple_set_args` | 여러 set 인자를 병합하여 필터링한다. |
| `next_filter_nonexistent_index_prints_no_matches` | 존재하지 않는 Index 는 "No matches" 를 출력한다. |
| `next_filter_set_with_invalid_silently_drops` | set 에 포함된 잘못된 값은 조용히 무시하고 유효한 값만 필터링한다. |
| `next_filter_set_with_zero_silently_drops` | set 내의 "0" 은 무시된다. |
| `all_invalid_single_prints_no_matches` | 단일 잘못된 인자는 "No matches" 를 출력한다. |
| `all_invalid_set_prints_no_matches` | 전체가 잘못된 set 은 "No matches" 를 출력한다. |
| `zero_bare_prints_no_matches` | bare "0" 인자는 "No matches" 를 출력한다. |

### `cli_info.rs`

| Test | 설명 |
| --- | --- |
| `info_single_index_renders_all_base_rows` | 단일 Index 인자로 info 를 실행하면 ID / Description / Status / Entered / Last modified / UID 행과 description("buy milk"), "Pending" 상태가 stdout 에 포함되고 stderr 은 비어 있다. |
| `info_omits_end_and_deleted_rows_for_pending` | pending 태스크의 info 렌더링에는 "End" 와 "Deleted" 행이 포함되지 않는다. |
| `info_renders_uid_row_with_valid_uid` | info 의 UID 행이 12자리 UID 를 담고 있으며, 해당 UID 로 재조회해도 동일한 태스크의 UID 와 description("buy milk") 이 렌더링된다. |
| `info_multiple_bare_args_renders_each_task` | 여러 bare Index 인자("1" "2") 로 조회하면 각 태스크의 description("one", "two") 이 모두 stdout 에 렌더링된다. |
| `info_nonexistent_index_prints_no_matches` | 존재하지 않는 Index("99") 로 조회하면 종료 코드 1 과 stderr 에 "No matches.\n" 을 출력한다. |
| `info_nonexistent_uid_prints_no_matches` | 존재하지 않는 UID("aaaaaaaaaaaa") 로 조회하면 종료 코드 1 과 stderr 에 "No matches.\n" 을 출력한다. |
| `info_empty_db_with_index_prints_no_matches` | 빈 DB 에서 Index("1") 로 조회하면 종료 코드 1 과 stderr 에 "No matches.\n" 을 출력한다. |
| `next_and_info_render_together_when_set_and_bare_given` | set("1,2") 과 bare("3") 가 함께 주어지면 "2 tasks" footer 를 가진 next 테이블이 먼저 렌더링되고 그 뒤에 info 테이블("Last modified") 이 이어서 렌더링되며 "one", "two", "three" 가 모두 stdout 에 포함된다. |
