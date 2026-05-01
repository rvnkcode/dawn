use dawn::domain::task::unique_id::{UID_LENGTH, UID_PATTERN};
use dawn::domain::task::{Description, Filter, Index, IndexRange, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

// Taskwarrior parity: indices reject leading zeros (e.g. "007" is text, not 7).
const INDEX_PATTERN: &str = r"[1-9]\d*";
const RANGE_PATTERN: &str = r"[1-9]\d*-[1-9]\d*";

static INDEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{INDEX_PATTERN}$")).unwrap());

// e.g. 1-10
static RANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{RANGE_PATTERN}$")).unwrap());

// e.g. 1,2-5,abcdefghi-_0
static SET_RE: LazyLock<Regex> = LazyLock::new(|| {
    let id_segment = format!(r"(?:{UID_PATTERN}|{INDEX_PATTERN}|{RANGE_PATTERN})");
    Regex::new(&format!(r"^{id_segment}(?:,{id_segment})+$")).unwrap()
});

/// List types to display when no command is present
#[derive(Debug, PartialEq)]
pub(crate) enum DefaultCommand {
    Next(Filter),
    Info(Filter),
}

/// No command word: pre→Filter, with bare-id heuristic for Info/Next routing
pub(crate) fn parse_default(raw_terms: &[String]) -> DefaultCommand {
    let (filter, has_bare_id) = classify(raw_terms);
    if has_bare_id {
        DefaultCommand::Info(filter)
    } else {
        DefaultCommand::Next(filter)
    }
}

/// Read-only report (e.g. `all`): pre and post merge into a single filter pass
pub(crate) fn parse_report(pre: &[String], post: &[String]) -> Filter {
    classify(pre.iter().chain(post.iter())).0
}

/// Mutation (modify/done/delete): pre→filter, post→description or annotation
pub(crate) fn parse_mutation(pre: &[String], post: &[String]) -> (Filter, Option<Description>) {
    let pre_filter = classify(pre).0;
    if pre_filter.is_empty() {
        promote_ids_from_post(post)
    } else {
        let desc: Vec<&str> = post
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let desc_opt = if desc.is_empty() {
            None
        } else {
            // non-empty by guard above; .ok() is None-as-unreachable
            Description::new(&desc.join(" ")).ok()
        };
        (pre_filter, desc_opt)
    }
}

fn classify<S: AsRef<str>>(raw_terms: impl IntoIterator<Item = S>) -> (Filter, bool) {
    let p = process_terms(raw_terms);
    let filter = Filter::default()
        .with_uids(p.uids)
        .with_indices(p.indices)
        .with_index_ranges(p.index_ranges)
        .with_words(p.words);
    (filter, p.has_bare_id)
}

// Promotes IDs from post into the filter when pre filter is empty;
// remaining words become the description or annotation
fn promote_ids_from_post(post: &[String]) -> (Filter, Option<Description>) {
    let parsed = process_terms(post);
    let filter = Filter::default()
        .with_uids(parsed.uids)
        .with_indices(parsed.indices)
        .with_index_ranges(parsed.index_ranges);
    let desc_opt = if parsed.words.is_empty() {
        None
    } else {
        // non-empty by guard above; .ok() is None-as-unreachable
        Description::new(&parsed.words.join(" ")).ok()
    };
    (filter, desc_opt)
}

#[derive(Default)]
struct Parsed {
    uids: HashSet<UniqueID>,
    indices: HashSet<Index>,
    index_ranges: HashSet<IndexRange>,
    words: Vec<String>,
    has_bare_id: bool,
}

fn process_terms<S: AsRef<str>>(raw_terms: impl IntoIterator<Item = S>) -> Parsed {
    let mut out = Parsed::default();

    for raw in raw_terms {
        let fragment = raw.as_ref().trim();
        if fragment.is_empty() {
            continue;
        }

        if SET_RE.is_match(fragment) {
            // Treat as a set of IDs: skip heuristic
            for seg in fragment.split(',') {
                if let Ok(u) = UniqueID::from_str(seg) {
                    out.uids.insert(u);
                } else if let Ok(i) = Index::from_str(seg) {
                    out.indices.insert(i);
                } else if RANGE_RE.is_match(seg) {
                    parse_range_segment(seg, &mut out);
                }
            }
            continue;
        }

        // UID before range: a 12-char digit/hyphen token (e.g. "12345-678901")
        // matches both; favor UID since such ranges need ~10-digit indices.
        if !looks_like_word(fragment) {
            if let Ok(u) = UniqueID::from_str(fragment) {
                out.uids.insert(u);
                out.has_bare_id = true;
                continue;
            }
            if INDEX_RE.is_match(fragment)
                && let Ok(i) = Index::from_str(fragment)
            {
                out.indices.insert(i);
                out.has_bare_id = true;
                continue;
            }
        }

        // Bare range (e.g. "5-10"): treated like a set, does not set has_bare_id.
        if RANGE_RE.is_match(fragment) {
            parse_range_segment(fragment, &mut out);
            continue;
        }

        // Fallback: treat as a search word
        out.words.push(fragment.to_string());
    }

    out
}

fn parse_range_segment(seg: &str, out: &mut Parsed) {
    let Some((lhs, rhs)) = seg.split_once('-') else {
        return;
    };
    let (Ok(start), Ok(end)) = (Index::from_str(lhs), Index::from_str(rhs)) else {
        return;
    };
    match IndexRange::new(start, end) {
        Ok(range) => {
            out.index_ranges.insert(range);
        }
        Err(idx) => {
            // Equal bounds (e.g. "5-5") collapse to a single index
            out.indices.insert(idx);
        }
    }
}

// Word-like heuristic: nanoid collision with this shape occurs at 40ppm (0.004%)
fn looks_like_word(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == UID_LENGTH
        && bytes[0].is_ascii_alphabetic()
        && bytes[1..].iter().all(|b| b.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(terms: &[&str]) -> Vec<String> {
        terms.iter().map(|s| s.to_string()).collect()
    }

    fn uid(s: &str) -> UniqueID {
        s.parse().unwrap()
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
    fn single_uid_bare_yields_info() {
        assert_eq!(
            parse_default(&raw(&["abcdefghi-_0"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("abcdefghi-_0")])),
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
    fn twelve_digit_numeric_parses_as_uid() {
        assert_eq!(
            parse_default(&raw(&["123456789012"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("123456789012")])),
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
    fn set_with_index_and_uid_yields_next() {
        assert_eq!(
            parse_default(&raw(&["1,abcdefghi-_0"])),
            DefaultCommand::Next(
                Filter::default()
                    .with_indices([idx(1)])
                    .with_uids([uid("abcdefghi-_0")]),
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
    fn invalid_bare_collected_as_word() {
        assert_eq!(
            parse_default(&raw(&["invalid"])),
            DefaultCommand::Next(Filter::default().with_words(["invalid"])),
        );
    }

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

    #[test]
    fn invalid_bare_mixed_with_set_collected_as_word() {
        assert_eq!(
            parse_default(&raw(&["invalid", "1,2"])),
            DefaultCommand::Next(
                Filter::default()
                    .with_indices([idx(1), idx(2)])
                    .with_words(["invalid"]),
            ),
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

    #[test]
    fn set_with_zero_segment_demoted_to_word() {
        // Strict SET_RE rejects "0" as an index segment, so the whole token
        // falls through and becomes a single word.
        assert_eq!(
            parse_default(&raw(&["1,0,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,0,2"])),
        );
    }

    // ── 12-letter heuristic: word-shaped fragments are demoted to words ──

    #[test]
    fn twelve_letter_lowercase_collected_as_word() {
        assert_eq!(
            parse_default(&raw(&["breakthrough"])),
            DefaultCommand::Next(Filter::default().with_words(["breakthrough"])),
        );
    }

    #[test]
    fn twelve_letter_title_case_collected_as_word() {
        assert_eq!(
            parse_default(&raw(&["Acknowledged"])),
            DefaultCommand::Next(Filter::default().with_words(["Acknowledged"])),
        );
    }

    #[test]
    fn twelve_letter_all_uppercase_unaffected_by_heuristic() {
        // ALL CAPS is intentionally outside the heuristic — treated as UID.
        assert_eq!(
            parse_default(&raw(&["BREAKTHROUGH"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("BREAKTHROUGH")])),
        );
    }

    #[test]
    fn uid_with_digit_unaffected_by_heuristic() {
        assert_eq!(
            parse_default(&raw(&["abc1efghijkl"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("abc1efghijkl")])),
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
    fn bare_double_zero_collected_as_word() {
        assert_eq!(
            parse_default(&raw(&["00"])),
            DefaultCommand::Next(Filter::default().with_words(["00"])),
        );
    }

    #[test]
    fn set_with_leading_zero_segment_demoted_to_word() {
        assert_eq!(
            parse_default(&raw(&["1,007"])),
            DefaultCommand::Next(Filter::default().with_words(["1,007"])),
        );
    }

    #[test]
    fn set_with_word_shaped_segment_treated_as_uid() {
        // Comma form is an explicit "this is an ID list" signal, so the word
        // heuristic is intentionally skipped inside sets. "breakthrough" is
        // parsed as a UID alongside index 1.
        assert_eq!(
            parse_default(&raw(&["breakthrough,1"])),
            DefaultCommand::Next(
                Filter::default()
                    .with_indices([idx(1)])
                    .with_uids([uid("breakthrough")]),
            ),
        );
    }

    // ── Index ranges ──

    #[test]
    fn twelve_char_digit_hyphen_token_parses_as_uid_not_range() {
        // Boundary: matches both UID_PATTERN and RANGE_RE — UID wins.
        assert_eq!(
            parse_default(&raw(&["12345-678901"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("12345-678901")])),
        );
    }

    #[test]
    fn bare_range_yields_next() {
        assert_eq!(
            parse_default(&raw(&["5-10"])),
            DefaultCommand::Next(Filter::default().with_index_ranges([range(5, 10)])),
        );
    }

    #[test]
    fn bare_range_alone_does_not_set_info() {
        // Regression guard: a lone range routes to Next, not Info.
        match parse_default(&raw(&["5-10"])) {
            DefaultCommand::Next(_) => {}
            DefaultCommand::Info(_) => panic!("bare range must route to Next"),
        }
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
    fn range_in_set_with_uid() {
        assert_eq!(
            parse_default(&raw(&["abcdefghi-_0,5-10"])),
            DefaultCommand::Next(
                Filter::default()
                    .with_uids([uid("abcdefghi-_0")])
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
    fn mutation_pre_index_post_word() {
        assert_eq!(
            parse_mutation(&raw(&["1"]), &raw(&["foo"])),
            (Filter::default().with_indices([idx(1)]), Some(desc("foo")),),
        );
    }

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

    #[test]
    fn mutation_pre_range_post_word() {
        assert_eq!(
            parse_mutation(&raw(&["5-10"]), &raw(&["foo"])),
            (
                Filter::default().with_index_ranges([range(5, 10)]),
                Some(desc("foo")),
            ),
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
    fn mutation_whitespace_only_pre_promotes_uid() {
        assert_eq!(
            parse_mutation(&raw(&["", "  "]), &raw(&["abcdefghi-_0", "new"])),
            (
                Filter::default().with_uids([uid("abcdefghi-_0")]),
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
    fn mutation_empty_pre_word_heuristic_demotion() {
        // 12-letter all-lowercase falls into the word heuristic → description
        assert_eq!(
            parse_mutation(&raw(&[]), &raw(&["breakthrough", "foo"])),
            (Filter::default(), Some(desc("breakthrough foo")),),
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
    fn report_uid_in_pre_word_in_post() {
        assert_eq!(
            parse_report(&raw(&["abcdefghi-_0"]), &raw(&["search"])),
            Filter::default()
                .with_uids([uid("abcdefghi-_0")])
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
}
