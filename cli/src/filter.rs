use dawn::domain::task::unique_id::{UID_LENGTH, UID_PATTERN};
use dawn::domain::task::{Description, Filter, Index, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

// Taskwarrior parity: indices reject leading zeros (e.g. "007" is text, not 7).
const INDEX_PATTERN: &str = r"[1-9]\d*";

static INDEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{INDEX_PATTERN}$")).unwrap());

static SET_RE: LazyLock<Regex> = LazyLock::new(|| {
    let id_segment = format!(r"(?:{UID_PATTERN}|{INDEX_PATTERN})");
    Regex::new(&format!(r"^{id_segment}(?:,{id_segment})+$")).unwrap()
});

#[derive(Debug, PartialEq)]
pub(crate) enum DefaultCommand {
    Next(Filter),
    Info(Filter),
}

// Default list; without any command input
pub(crate) fn parse_and_determine_command(raw_terms: &[String]) -> DefaultCommand {
    match classify(raw_terms) {
        (filter, true) => DefaultCommand::Info(filter),
        (filter, false) => DefaultCommand::Next(filter),
    }
}

pub(crate) fn parse_from_mods(pre: &[String], mods: &[String]) -> (Filter, Option<Description>) {
    let pre_filter = parse(pre);
    if pre_filter.is_empty() {
        promote_ids_from_mods(mods)
    } else {
        // join mods args input into a new description
        let desc: Vec<&str> = mods
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

fn parse(raw_terms: &[String]) -> Filter {
    classify(raw_terms).0
}

fn classify(raw_terms: &[String]) -> (Filter, bool) {
    let p = process_terms(raw_terms);
    let filter = Filter::default()
        .with_uids(p.uids)
        .with_indices(p.indices)
        .with_words(p.words);
    (filter, p.has_bare_id)
}

// Parse ID filters from modifications, promoting to filter if pre-command filter is empty
fn promote_ids_from_mods(mods: &[String]) -> (Filter, Option<Description>) {
    let parsed = process_terms(mods);
    let filter = Filter::default()
        .with_uids(parsed.uids)
        .with_indices(parsed.indices);
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
    words: Vec<String>,
    has_bare_id: bool,
}

fn process_terms(raw_terms: &[String]) -> Parsed {
    let mut out = Parsed::default();

    for raw in raw_terms {
        let fragment = raw.trim();
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
                }
            }
            continue;
        }

        // Bare IDs: for info table
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

        // Fallback: treat as a search word
        out.words.push(fragment.to_string());
    }

    out
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

    // ── Bare (single token, no comma) → Info ──

    #[test]
    fn single_uid_bare_yields_info() {
        assert_eq!(
            parse_and_determine_command(&raw(&["abcdefghi-_0"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("abcdefghi-_0")])),
        );
    }

    #[test]
    fn single_index_bare_yields_info() {
        assert_eq!(
            parse_and_determine_command(&raw(&["42"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(42)])),
        );
    }

    #[test]
    fn twelve_digit_numeric_parses_as_uid() {
        assert_eq!(
            parse_and_determine_command(&raw(&["123456789012"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("123456789012")])),
        );
    }

    #[test]
    fn multiple_bare_args_merge_and_yield_info() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1", "2"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Set (comma-separated) → Next ──

    #[test]
    fn comma_separated_indices_yield_next() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1,2,3"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
        );
    }

    #[test]
    fn set_with_index_and_uid_yields_next() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1,abcdefghi-_0"])),
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
            parse_and_determine_command(&raw(&["1,2", "2,3"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
        );
    }

    #[test]
    fn duplicates_within_set_are_deduped() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1,1,1"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1)])),
        );
    }

    // ── Mixed bare + set → Info (UNION merge) ──

    #[test]
    fn bare_plus_set_yields_info_with_union() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1", "2,3"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
        );
    }

    #[test]
    fn set_and_bare_with_overlapping_ids_dedup() {
        // Bare "2" overlaps with set "2" — deduped to {1, 2}.
        assert_eq!(
            parse_and_determine_command(&raw(&["1,2", "2"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Non-ID bare → collected as word, does not flip to Info ──

    #[test]
    fn invalid_bare_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["invalid"])),
            DefaultCommand::Next(Filter::default().with_words(["invalid"])),
        );
    }

    #[test]
    fn zero_bare_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["0"])),
            DefaultCommand::Next(Filter::default().with_words(["0"])),
        );
    }

    #[test]
    fn non_ascii_bare_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["한국어"])),
            DefaultCommand::Next(Filter::default().with_words(["한국어"])),
        );
    }

    #[test]
    fn invalid_bare_mixed_with_set_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["invalid", "1,2"])),
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
            parse_and_determine_command(&raw(&["1,invalid,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,invalid,2"])),
        );
    }

    #[test]
    fn all_invalid_set_demotes_to_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["invalid,xyz"])),
            DefaultCommand::Next(Filter::default().with_words(["invalid,xyz"])),
        );
    }

    // ── Malformed comma shapes → whole token demoted to a single word ──

    #[test]
    fn empty_string_yields_next_with_empty_filter() {
        assert_eq!(
            parse_and_determine_command(&raw(&[""])),
            DefaultCommand::Next(Filter::default())
        );
    }

    #[test]
    fn double_comma_demotes_to_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1,,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,,2"])),
        );
    }

    #[test]
    fn trailing_comma_demotes_to_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1,"])),
            DefaultCommand::Next(Filter::default().with_words(["1,"])),
        );
    }

    #[test]
    fn leading_comma_demotes_to_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&[",1"])),
            DefaultCommand::Next(Filter::default().with_words([",1"])),
        );
    }

    #[test]
    fn whitespace_around_comma_demotes_to_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1 , 2"])),
            DefaultCommand::Next(Filter::default().with_words(["1 , 2"])),
        );
    }

    #[test]
    fn outer_whitespace_trimmed_then_parsed_as_set() {
        assert_eq!(
            parse_and_determine_command(&raw(&["  1,2  "])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Words filter (search terms) ──

    #[test]
    fn bare_word_collected_into_words() {
        assert_eq!(
            parse_and_determine_command(&raw(&["hello"])),
            DefaultCommand::Next(Filter::default().with_words(["hello"])),
        );
    }

    #[test]
    fn multiple_bare_words_collected() {
        assert_eq!(
            parse_and_determine_command(&raw(&["hello", "world"])),
            DefaultCommand::Next(Filter::default().with_words(["hello", "world"])),
        );
    }

    #[test]
    fn bare_id_with_word_yields_info_with_word_filter() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1", "hello"])),
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
            parse_and_determine_command(&raw(&["1,2", "hello"])),
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
            parse_and_determine_command(&raw(&["hello", "hello"])),
            DefaultCommand::Next(Filter::default().with_words(["hello"])),
        );
    }

    #[test]
    fn surrounding_whitespace_on_word_trimmed() {
        assert_eq!(
            parse_and_determine_command(&raw(&["  hello  "])),
            DefaultCommand::Next(Filter::default().with_words(["hello"])),
        );
    }

    #[test]
    fn set_with_zero_segment_demoted_to_word() {
        // Strict SET_RE rejects "0" as an index segment, so the whole token
        // falls through and becomes a single word.
        assert_eq!(
            parse_and_determine_command(&raw(&["1,0,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,0,2"])),
        );
    }

    // ── 12-letter heuristic: word-shaped fragments are demoted to words ──

    #[test]
    fn twelve_letter_lowercase_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["breakthrough"])),
            DefaultCommand::Next(Filter::default().with_words(["breakthrough"])),
        );
    }

    #[test]
    fn twelve_letter_title_case_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["Acknowledged"])),
            DefaultCommand::Next(Filter::default().with_words(["Acknowledged"])),
        );
    }

    #[test]
    fn twelve_letter_all_uppercase_unaffected_by_heuristic() {
        // ALL CAPS is intentionally outside the heuristic — treated as UID.
        assert_eq!(
            parse_and_determine_command(&raw(&["BREAKTHROUGH"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("BREAKTHROUGH")])),
        );
    }

    #[test]
    fn uid_with_digit_unaffected_by_heuristic() {
        assert_eq!(
            parse_and_determine_command(&raw(&["abc1efghijkl"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("abc1efghijkl")])),
        );
    }

    // ── Leading-zero numbers: Taskwarrior parity (007 is text, not 7) ──

    #[test]
    fn bare_leading_zero_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["007"])),
            DefaultCommand::Next(Filter::default().with_words(["007"])),
        );
    }

    #[test]
    fn bare_double_zero_collected_as_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["00"])),
            DefaultCommand::Next(Filter::default().with_words(["00"])),
        );
    }

    #[test]
    fn set_with_leading_zero_segment_demoted_to_word() {
        assert_eq!(
            parse_and_determine_command(&raw(&["1,007"])),
            DefaultCommand::Next(Filter::default().with_words(["1,007"])),
        );
    }

    #[test]
    fn set_with_word_shaped_segment_treated_as_uid() {
        // Comma form is an explicit "this is an ID list" signal, so the word
        // heuristic is intentionally skipped inside sets. "breakthrough" is
        // parsed as a UID alongside index 1.
        assert_eq!(
            parse_and_determine_command(&raw(&["breakthrough,1"])),
            DefaultCommand::Next(
                Filter::default()
                    .with_indices([idx(1)])
                    .with_uids([uid("breakthrough")]),
            ),
        );
    }

    // ── Empty input ──

    #[test]
    fn empty_input_yields_next_with_empty_filter() {
        assert_eq!(
            parse_and_determine_command(&raw(&[])),
            DefaultCommand::Next(Filter::default())
        );
    }

    // ── parse_modification: 비-promotion (pre 비어있지 않음) ──

    #[test]
    fn parse_modification_pre_index_mods_word() {
        assert_eq!(
            parse_from_mods(&raw(&["1"]), &raw(&["foo"])),
            (Filter::default().with_indices([idx(1)]), Some(desc("foo")),),
        );
    }

    #[test]
    fn parse_modification_pre_index_mods_id_shaped_stays_in_description() {
        // pre가 비어있지 않으므로 promotion 발동하지 않고, post의 "2"도 description으로 합쳐짐
        assert_eq!(
            parse_from_mods(&raw(&["1"]), &raw(&["2", "foo"])),
            (
                Filter::default().with_indices([idx(1)]),
                Some(desc("2 foo")),
            ),
        );
    }

    #[test]
    fn parse_modification_pre_word_mods_word() {
        assert_eq!(
            parse_from_mods(&raw(&["hello"]), &raw(&["foo"])),
            (Filter::default().with_words(["hello"]), Some(desc("foo")),),
        );
    }

    #[test]
    fn parse_modification_pre_index_mods_empty_yields_no_description() {
        assert_eq!(
            parse_from_mods(&raw(&["1"]), &raw(&[])),
            (Filter::default().with_indices([idx(1)]), None),
        );
    }

    // ── parse_modification: promotion (pre가 빈 vec 또는 빈 string만) ──

    #[test]
    fn parse_modification_empty_pre_promotes_index() {
        assert_eq!(
            parse_from_mods(&raw(&[]), &raw(&["1", "foo"])),
            (Filter::default().with_indices([idx(1)]), Some(desc("foo")),),
        );
    }

    #[test]
    fn parse_modification_blank_pre_strings_treated_as_empty() {
        // raw Vec은 비어있지 않지만 trim 결과 empty fragment만 → promotion 발동
        assert_eq!(
            parse_from_mods(&raw(&[""]), &raw(&["1", "foo"])),
            (Filter::default().with_indices([idx(1)]), Some(desc("foo")),),
        );
    }

    #[test]
    fn parse_modification_whitespace_only_pre_promotes_uid() {
        assert_eq!(
            parse_from_mods(&raw(&["", "  "]), &raw(&["abcdefghi-_0", "new"])),
            (
                Filter::default().with_uids([uid("abcdefghi-_0")]),
                Some(desc("new")),
            ),
        );
    }

    #[test]
    fn parse_modification_empty_pre_promotes_set() {
        assert_eq!(
            parse_from_mods(&raw(&[]), &raw(&["1,2", "foo"])),
            (
                Filter::default().with_indices([idx(1), idx(2)]),
                Some(desc("foo")),
            ),
        );
    }

    #[test]
    fn parse_modification_empty_pre_word_only_does_not_promote() {
        // taskwarrior parity: `task modify text modification`은 모두 description
        assert_eq!(
            parse_from_mods(&raw(&[]), &raw(&["text", "modification"])),
            (Filter::default(), Some(desc("text modification"))),
        );
    }

    #[test]
    fn parse_modification_empty_pre_leading_zero_treated_as_word() {
        assert_eq!(
            parse_from_mods(&raw(&[]), &raw(&["007", "foo"])),
            (Filter::default(), Some(desc("007 foo"))),
        );
    }

    #[test]
    fn parse_modification_empty_pre_word_heuristic_demotion() {
        // 12-letter all-lowercase는 word heuristic으로 description
        assert_eq!(
            parse_from_mods(&raw(&[]), &raw(&["breakthrough", "foo"])),
            (Filter::default(), Some(desc("breakthrough foo")),),
        );
    }

    #[test]
    fn parse_modification_empty_pre_and_mods() {
        assert_eq!(
            parse_from_mods(&raw(&[]), &raw(&[])),
            (Filter::default(), None),
        );
    }
}
