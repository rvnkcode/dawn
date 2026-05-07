use super::*;

fn raw(terms: &[&str]) -> Vec<String> {
    terms.iter().map(|s| s.to_string()).collect()
}

fn uuid(s: &str) -> UuidPrefix {
    UuidPrefix::parse(s).unwrap()
}

fn idx(n: usize) -> Index {
    Index::new(n).unwrap()
}

fn desc(s: &str) -> Description {
    Description::new(s).unwrap()
}

fn range(a: usize, b: usize) -> IndexRange {
    IndexRange::new(idx(a), idx(b)).unwrap()
}

// ── Bare (single token, no comma) → Info ──

#[test]
fn single_uuid_bare_yields_info() {
    assert_eq!(
        parse_default(&raw(&["550e8400-e29b-41d4-a716-446655440000"])),
        DefaultCommand::Info(
            Filter::default().with_uuids([uuid("550e8400-e29b-41d4-a716-446655440000")])
        ),
    );
}

// ── 8+ char hex prefix matches as UUID ──

#[test]
fn eight_char_hex_prefix_yields_info_as_uuid() {
    assert_eq!(
        parse_default(&raw(&["550e8400"])),
        DefaultCommand::Info(Filter::default().with_uuids([uuid("550e8400")])),
    );
}

#[test]
fn nine_char_uuid_with_hyphen_yields_info_as_uuid() {
    assert_eq!(
        parse_default(&raw(&["550e8400-"])),
        DefaultCommand::Info(Filter::default().with_uuids([uuid("550e8400-")])),
    );
}

#[test]
fn thirteen_char_first_two_groups_yields_info_as_uuid() {
    assert_eq!(
        parse_default(&raw(&["550e8400-e29b"])),
        DefaultCommand::Info(Filter::default().with_uuids([uuid("550e8400-e29b")])),
    );
}

#[test]
fn seven_char_hex_falls_to_word() {
    // Below TW's 8-char minimum → not a UUID.
    assert_eq!(
        parse_default(&raw(&["550e840"])),
        DefaultCommand::Next(Filter::default().with_words(["550e840"])),
    );
}

#[test]
fn malformed_uuid_with_oversized_group_falls_to_word() {
    // Second group overruns 4-char limit → invalid prefix shape.
    assert_eq!(
        parse_default(&raw(&["550e8400-e29b12"])),
        DefaultCommand::Next(Filter::default().with_words(["550e8400-e29b12"])),
    );
}

#[test]
fn single_index_bare_yields_info() {
    assert_eq!(
        parse_default(&raw(&["42"])),
        DefaultCommand::Info(Filter::default().with_indices([idx(42)])),
    );
}

#[test]
fn multiple_bare_args_merge_and_yield_info() {
    assert_eq!(
        parse_default(&raw(&["1", "2"])),
        DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2)])),
    );
}

// ── Set (comma-separated) → Next ──

#[test]
fn comma_separated_indices_yield_next() {
    assert_eq!(
        parse_default(&raw(&["1,2,3"])),
        DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
    );
}

#[test]
fn set_with_index_and_uuid_yields_next() {
    assert_eq!(
        parse_default(&raw(&["1,550e8400-e29b-41d4-a716-446655440000"])),
        DefaultCommand::Next(
            Filter::default()
                .with_indices([idx(1)])
                .with_uuids([uuid("550e8400-e29b-41d4-a716-446655440000")]),
        ),
    );
}

#[test]
fn uuid_prefix_in_set_treated_as_uuid() {
    assert_eq!(
        parse_default(&raw(&["1,550e8400"])),
        DefaultCommand::Next(
            Filter::default()
                .with_indices([idx(1)])
                .with_uuids([uuid("550e8400")]),
        ),
    );
}

#[test]
fn multiple_set_args_merge_and_dedup() {
    // Overlapping "2" is deduped by HashSet.
    assert_eq!(
        parse_default(&raw(&["1,2", "2,3"])),
        DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
    );
}

#[test]
fn duplicates_within_set_are_deduped() {
    assert_eq!(
        parse_default(&raw(&["1,1,1"])),
        DefaultCommand::Next(Filter::default().with_indices([idx(1)])),
    );
}

// ── Mixed bare + set → Info (UNION merge) ──

#[test]
fn bare_plus_set_yields_info_with_union() {
    assert_eq!(
        parse_default(&raw(&["1", "2,3"])),
        DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
    );
}

#[test]
fn set_and_bare_with_overlapping_ids_dedup() {
    // Bare "2" overlaps with set "2" — deduped to {1, 2}.
    assert_eq!(
        parse_default(&raw(&["1,2", "2"])),
        DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2)])),
    );
}

// ── Non-ID bare → collected as word, does not flip to Info ──

#[test]
fn zero_bare_collected_as_word() {
    assert_eq!(
        parse_default(&raw(&["0"])),
        DefaultCommand::Next(Filter::default().with_words(["0"])),
    );
}

#[test]
fn non_ascii_bare_collected_as_word() {
    assert_eq!(
        parse_default(&raw(&["한국어"])),
        DefaultCommand::Next(Filter::default().with_words(["한국어"])),
    );
}

// ── Invalid segment inside a set → whole token demoted to a single word ──

#[test]
fn invalid_segment_in_set_demotes_whole_token_to_word() {
    assert_eq!(
        parse_default(&raw(&["1,invalid,2"])),
        DefaultCommand::Next(Filter::default().with_words(["1,invalid,2"])),
    );
}

#[test]
fn all_invalid_set_demotes_to_word() {
    assert_eq!(
        parse_default(&raw(&["invalid,xyz"])),
        DefaultCommand::Next(Filter::default().with_words(["invalid,xyz"])),
    );
}

// ── Malformed comma shapes → whole token demoted to a single word ──

#[test]
fn empty_string_yields_next_with_empty_filter() {
    assert_eq!(
        parse_default(&raw(&[""])),
        DefaultCommand::Next(Filter::default())
    );
}

#[test]
fn double_comma_demotes_to_word() {
    assert_eq!(
        parse_default(&raw(&["1,,2"])),
        DefaultCommand::Next(Filter::default().with_words(["1,,2"])),
    );
}

#[test]
fn trailing_comma_demotes_to_word() {
    assert_eq!(
        parse_default(&raw(&["1,"])),
        DefaultCommand::Next(Filter::default().with_words(["1,"])),
    );
}

#[test]
fn leading_comma_demotes_to_word() {
    assert_eq!(
        parse_default(&raw(&[",1"])),
        DefaultCommand::Next(Filter::default().with_words([",1"])),
    );
}

#[test]
fn whitespace_around_comma_demotes_to_word() {
    assert_eq!(
        parse_default(&raw(&["1 , 2"])),
        DefaultCommand::Next(Filter::default().with_words(["1 , 2"])),
    );
}

#[test]
fn outer_whitespace_trimmed_then_parsed_as_set() {
    assert_eq!(
        parse_default(&raw(&["  1,2  "])),
        DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2)])),
    );
}

// ── Words filter (search terms) ──

#[test]
fn bare_word_collected_into_words() {
    assert_eq!(
        parse_default(&raw(&["hello"])),
        DefaultCommand::Next(Filter::default().with_words(["hello"])),
    );
}

#[test]
fn multiple_bare_words_collected() {
    assert_eq!(
        parse_default(&raw(&["hello", "world"])),
        DefaultCommand::Next(Filter::default().with_words(["hello", "world"])),
    );
}

#[test]
fn bare_id_with_word_yields_info_with_word_filter() {
    assert_eq!(
        parse_default(&raw(&["1", "hello"])),
        DefaultCommand::Info(
            Filter::default()
                .with_indices([idx(1)])
                .with_words(["hello"]),
        ),
    );
}

#[test]
fn set_with_word_yields_next_with_both() {
    assert_eq!(
        parse_default(&raw(&["1,2", "hello"])),
        DefaultCommand::Next(
            Filter::default()
                .with_indices([idx(1), idx(2)])
                .with_words(["hello"]),
        ),
    );
}

#[test]
fn duplicate_words_deduped() {
    assert_eq!(
        parse_default(&raw(&["hello", "hello"])),
        DefaultCommand::Next(Filter::default().with_words(["hello"])),
    );
}

#[test]
fn surrounding_whitespace_on_word_trimmed() {
    assert_eq!(
        parse_default(&raw(&["  hello  "])),
        DefaultCommand::Next(Filter::default().with_words(["hello"])),
    );
}

// ── Leading-zero numbers: Taskwarrior parity (007 is text, not 7) ──

#[test]
fn bare_leading_zero_collected_as_word() {
    assert_eq!(
        parse_default(&raw(&["007"])),
        DefaultCommand::Next(Filter::default().with_words(["007"])),
    );
}

#[test]
fn set_with_leading_zero_segment_demoted_to_word() {
    assert_eq!(
        parse_default(&raw(&["1,007"])),
        DefaultCommand::Next(Filter::default().with_words(["1,007"])),
    );
}

// ── Index ranges ──

#[test]
fn bare_range_yields_next() {
    assert_eq!(
        parse_default(&raw(&["5-10"])),
        DefaultCommand::Next(Filter::default().with_index_ranges([range(5, 10)])),
    );
}

#[test]
fn bare_range_descending_swaps() {
    assert_eq!(
        parse_default(&raw(&["10-5"])),
        DefaultCommand::Next(Filter::default().with_index_ranges([range(5, 10)])),
    );
}

#[test]
fn range_equal_bounds_collapses_to_index() {
    assert_eq!(
        parse_default(&raw(&["5-5"])),
        DefaultCommand::Next(Filter::default().with_indices([idx(5)])),
    );
}

#[test]
fn range_with_invalid_right_falls_to_word() {
    assert_eq!(
        parse_default(&raw(&["1-foo"])),
        DefaultCommand::Next(Filter::default().with_words(["1-foo"])),
    );
}

#[test]
fn range_with_invalid_left_falls_to_word() {
    assert_eq!(
        parse_default(&raw(&["foo-1"])),
        DefaultCommand::Next(Filter::default().with_words(["foo-1"])),
    );
}

#[test]
fn range_with_zero_left_falls_to_word() {
    assert_eq!(
        parse_default(&raw(&["0-5"])),
        DefaultCommand::Next(Filter::default().with_words(["0-5"])),
    );
}

#[test]
fn range_with_zero_right_falls_to_word() {
    assert_eq!(
        parse_default(&raw(&["1-0"])),
        DefaultCommand::Next(Filter::default().with_words(["1-0"])),
    );
}

#[test]
fn open_range_left_falls_to_word() {
    assert_eq!(
        parse_default(&raw(&["5-"])),
        DefaultCommand::Next(Filter::default().with_words(["5-"])),
    );
}

#[test]
fn open_range_right_falls_to_word() {
    assert_eq!(
        parse_default(&raw(&["-10"])),
        DefaultCommand::Next(Filter::default().with_words(["-10"])),
    );
}

#[test]
fn range_in_set_with_indices() {
    assert_eq!(
        parse_default(&raw(&["1,3,5-10,19"])),
        DefaultCommand::Next(
            Filter::default()
                .with_indices([idx(1), idx(3), idx(19)])
                .with_index_ranges([range(5, 10)]),
        ),
    );
}

#[test]
fn range_in_set_with_uuid() {
    assert_eq!(
        parse_default(&raw(&["550e8400-e29b-41d4-a716-446655440000,5-10"])),
        DefaultCommand::Next(
            Filter::default()
                .with_uuids([uuid("550e8400-e29b-41d4-a716-446655440000")])
                .with_index_ranges([range(5, 10)]),
        ),
    );
}

#[test]
fn multiple_bare_ranges_dedup() {
    assert_eq!(
        parse_default(&raw(&["5-10", "5-10"])),
        DefaultCommand::Next(Filter::default().with_index_ranges([range(5, 10)])),
    );
}

#[test]
fn bare_range_plus_bare_index_yields_info() {
    assert_eq!(
        parse_default(&raw(&["1", "5-10"])),
        DefaultCommand::Info(
            Filter::default()
                .with_indices([idx(1)])
                .with_index_ranges([range(5, 10)]),
        ),
    );
}

#[test]
fn range_in_set_with_invalid_segment_demotes_to_word() {
    assert_eq!(
        parse_default(&raw(&["1,5-foo,2"])),
        DefaultCommand::Next(Filter::default().with_words(["1,5-foo,2"])),
    );
}

// ── parse_range_segment defensive guards ──
// Direct calls bypass the RANGE_RE precondition that public callers enforce,
// exercising the early-return branches that protect against future misuse.

#[test]
fn parse_range_segment_without_hyphen_is_noop() {
    let mut out = Parsed::default();
    parse_range_segment("5", &mut out);
    assert!(out.indices.is_empty());
    assert!(out.index_ranges.is_empty());
}

#[test]
fn parse_range_segment_with_overflow_bound_is_noop() {
    let mut out = Parsed::default();
    parse_range_segment("99999999999999999999-1", &mut out);
    assert!(out.indices.is_empty());
    assert!(out.index_ranges.is_empty());
}

// ── Empty input ──

#[test]
fn empty_input_yields_next_with_empty_filter() {
    assert_eq!(
        parse_default(&raw(&[])),
        DefaultCommand::Next(Filter::default())
    );
}

// ── mutation: no promotion (pre is non-empty) ──
#[test]
fn mutation_pre_index_post_id_shaped_stays_in_description() {
    // pre is non-empty, so promotion does not trigger; post's "2" is folded into the description
    assert_eq!(
        parse_mutation(&raw(&["1"]), &raw(&["2", "foo"])),
        (
            Filter::default().with_indices([idx(1)]),
            Some(desc("2 foo")),
        ),
    );
}

#[test]
fn mutation_pre_word_post_word() {
    assert_eq!(
        parse_mutation(&raw(&["hello"]), &raw(&["foo"])),
        (Filter::default().with_words(["hello"]), Some(desc("foo")),),
    );
}

#[test]
fn mutation_pre_index_post_empty_yields_no_description() {
    assert_eq!(
        parse_mutation(&raw(&["1"]), &raw(&[])),
        (Filter::default().with_indices([idx(1)]), None),
    );
}

// ── mutation: promotion (pre is empty vec or only blank strings) ──

#[test]
fn mutation_empty_pre_promotes_index() {
    assert_eq!(
        parse_mutation(&raw(&[]), &raw(&["1", "foo"])),
        (Filter::default().with_indices([idx(1)]), Some(desc("foo")),),
    );
}

#[test]
fn mutation_blank_pre_strings_treated_as_empty() {
    // raw Vec is non-empty but trim leaves only empty fragments → promotion triggers
    assert_eq!(
        parse_mutation(&raw(&[""]), &raw(&["1", "foo"])),
        (Filter::default().with_indices([idx(1)]), Some(desc("foo")),),
    );
}

#[test]
fn mutation_whitespace_only_pre_promotes_uuid() {
    assert_eq!(
        parse_mutation(
            &raw(&["", "  "]),
            &raw(&["550e8400-e29b-41d4-a716-446655440000", "new"])
        ),
        (
            Filter::default().with_uuids([uuid("550e8400-e29b-41d4-a716-446655440000")]),
            Some(desc("new")),
        ),
    );
}

#[test]
fn mutation_empty_pre_promotes_set() {
    assert_eq!(
        parse_mutation(&raw(&[]), &raw(&["1,2", "foo"])),
        (
            Filter::default().with_indices([idx(1), idx(2)]),
            Some(desc("foo")),
        ),
    );
}

#[test]
fn mutation_empty_pre_word_only_does_not_promote() {
    // taskwarrior parity: `task modify text modification` — all goes to description
    assert_eq!(
        parse_mutation(&raw(&[]), &raw(&["text", "modification"])),
        (Filter::default(), Some(desc("text modification"))),
    );
}

#[test]
fn mutation_empty_pre_leading_zero_treated_as_word() {
    assert_eq!(
        parse_mutation(&raw(&[]), &raw(&["007", "foo"])),
        (Filter::default(), Some(desc("007 foo"))),
    );
}

#[test]
fn mutation_empty_pre_and_post() {
    assert_eq!(
        parse_mutation(&raw(&[]), &raw(&[])),
        (Filter::default(), None),
    );
}

#[test]
fn mutation_empty_pre_promotes_range() {
    assert_eq!(
        parse_mutation(&raw(&[]), &raw(&["5-10", "foo"])),
        (
            Filter::default().with_index_ranges([range(5, 10)]),
            Some(desc("foo")),
        ),
    );
}

// ── report: both pre and post are treated as filter ──

#[test]
fn report_pre_only() {
    assert_eq!(
        parse_report(&raw(&["1", "hello"]), &raw(&[])),
        Filter::default()
            .with_indices([idx(1)])
            .with_words(["hello"]),
    );
}

#[test]
fn report_post_only() {
    assert_eq!(
        parse_report(&raw(&[]), &raw(&["1", "hello"])),
        Filter::default()
            .with_indices([idx(1)])
            .with_words(["hello"]),
    );
}

#[test]
fn report_merges_indices_from_both_sides() {
    assert_eq!(
        parse_report(&raw(&["1"]), &raw(&["2"])),
        Filter::default().with_indices([idx(1), idx(2)]),
    );
}

#[test]
fn report_merges_words_from_both_sides() {
    assert_eq!(
        parse_report(&raw(&["hello"]), &raw(&["world"])),
        Filter::default().with_words(["hello", "world"]),
    );
}

#[test]
fn report_dedupes_across_sides() {
    assert_eq!(
        parse_report(&raw(&["1"]), &raw(&["1"])),
        Filter::default().with_indices([idx(1)]),
    );
}

#[test]
fn report_set_in_post() {
    assert_eq!(
        parse_report(&raw(&[]), &raw(&["1,2,3"])),
        Filter::default().with_indices([idx(1), idx(2), idx(3)]),
    );
}

#[test]
fn report_both_empty_yields_empty_filter() {
    assert_eq!(parse_report(&raw(&[]), &raw(&[])), Filter::default());
}

#[test]
fn report_uuid_in_pre_word_in_post() {
    assert_eq!(
        parse_report(
            &raw(&["550e8400-e29b-41d4-a716-446655440000"]),
            &raw(&["search"])
        ),
        Filter::default()
            .with_uuids([uuid("550e8400-e29b-41d4-a716-446655440000")])
            .with_words(["search"]),
    );
}

#[test]
fn report_set_with_range_in_post() {
    assert_eq!(
        parse_report(&raw(&[]), &raw(&["1,5-10"])),
        Filter::default()
            .with_indices([idx(1)])
            .with_index_ranges([range(5, 10)]),
    );
}
