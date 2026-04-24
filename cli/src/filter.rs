use dawn::domain::task::{Filter, Index, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

const ID_SEGMENT: &str = r"(?:[A-Za-z0-9_-]{12}|0*[1-9]\d*)";

static SET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"^{ID_SEGMENT}(?:,{ID_SEGMENT})+$")).unwrap());

#[derive(Debug, PartialEq)]
pub(crate) enum DefaultCommand {
    Next(Filter),
    Info(Filter),
}

pub(crate) fn parse(raw_terms: &[String]) -> DefaultCommand {
    let mut uids = HashSet::new();
    let mut indices = HashSet::new();
    let mut words: Vec<String> = Vec::new();
    let mut has_bare_id = false;

    for raw in raw_terms {
        let fragment = raw.trim();
        if fragment.is_empty() {
            continue;
        }

        if SET_RE.is_match(fragment) {
            for seg in fragment.split(',') {
                if let Ok(u) = UniqueID::from_str(seg) {
                    uids.insert(u);
                } else if let Ok(i) = Index::from_str(seg) {
                    indices.insert(i);
                }
            }
        } else if let Ok(u) = UniqueID::from_str(fragment) {
            uids.insert(u);
            has_bare_id = true;
        } else if let Ok(i) = Index::from_str(fragment) {
            indices.insert(i);
            has_bare_id = true;
        } else {
            words.push(fragment.to_string());
        }
    }

    let filter = Filter::default()
        .with_uids(uids)
        .with_indices(indices)
        .with_words(words);
    if has_bare_id {
        DefaultCommand::Info(filter)
    } else {
        DefaultCommand::Next(filter)
    }
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

    // ── Bare (single token, no comma) → Info ──

    #[test]
    fn single_uid_bare_yields_info() {
        assert_eq!(
            parse(&raw(&["abcdefghijkl"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("abcdefghijkl")])),
        );
    }

    #[test]
    fn single_index_bare_yields_info() {
        assert_eq!(
            parse(&raw(&["42"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(42)])),
        );
    }

    #[test]
    fn twelve_digit_numeric_parses_as_uid() {
        assert_eq!(
            parse(&raw(&["123456789012"])),
            DefaultCommand::Info(Filter::default().with_uids([uid("123456789012")])),
        );
    }

    #[test]
    fn multiple_bare_args_merge_and_yield_info() {
        assert_eq!(
            parse(&raw(&["1", "2"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Set (comma-separated) → Next ──

    #[test]
    fn comma_separated_indices_yield_next() {
        assert_eq!(
            parse(&raw(&["1,2,3"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
        );
    }

    #[test]
    fn set_with_index_and_uid_yields_next() {
        assert_eq!(
            parse(&raw(&["1,abcdefghijkl"])),
            DefaultCommand::Next(
                Filter::default()
                    .with_indices([idx(1)])
                    .with_uids([uid("abcdefghijkl")]),
            ),
        );
    }

    #[test]
    fn multiple_set_args_merge_and_dedup() {
        // Overlapping "2" is deduped by HashSet.
        assert_eq!(
            parse(&raw(&["1,2", "2,3"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
        );
    }

    #[test]
    fn duplicates_within_set_are_deduped() {
        assert_eq!(
            parse(&raw(&["1,1,1"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1)])),
        );
    }

    // ── Mixed bare + set → Info (UNION merge) ──

    #[test]
    fn bare_plus_set_yields_info_with_union() {
        assert_eq!(
            parse(&raw(&["1", "2,3"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2), idx(3)])),
        );
    }

    #[test]
    fn set_and_bare_with_overlapping_ids_dedup() {
        // Bare "2" overlaps with set "2" — deduped to {1, 2}.
        assert_eq!(
            parse(&raw(&["1,2", "2"])),
            DefaultCommand::Info(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Non-ID bare → collected as word, does not flip to Info ──

    #[test]
    fn invalid_bare_collected_as_word() {
        assert_eq!(
            parse(&raw(&["invalid"])),
            DefaultCommand::Next(Filter::default().with_words(["invalid"])),
        );
    }

    #[test]
    fn zero_bare_collected_as_word() {
        assert_eq!(
            parse(&raw(&["0"])),
            DefaultCommand::Next(Filter::default().with_words(["0"])),
        );
    }

    #[test]
    fn non_ascii_bare_collected_as_word() {
        assert_eq!(
            parse(&raw(&["한국어"])),
            DefaultCommand::Next(Filter::default().with_words(["한국어"])),
        );
    }

    #[test]
    fn invalid_bare_mixed_with_set_collected_as_word() {
        assert_eq!(
            parse(&raw(&["invalid", "1,2"])),
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
            parse(&raw(&["1,invalid,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,invalid,2"])),
        );
    }

    #[test]
    fn all_invalid_set_demotes_to_word() {
        assert_eq!(
            parse(&raw(&["invalid,xyz"])),
            DefaultCommand::Next(Filter::default().with_words(["invalid,xyz"])),
        );
    }

    // ── Malformed comma shapes → whole token demoted to a single word ──

    #[test]
    fn empty_string_yields_next_with_empty_filter() {
        assert_eq!(parse(&raw(&[""])), DefaultCommand::Next(Filter::default()));
    }

    #[test]
    fn double_comma_demotes_to_word() {
        assert_eq!(
            parse(&raw(&["1,,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,,2"])),
        );
    }

    #[test]
    fn trailing_comma_demotes_to_word() {
        assert_eq!(
            parse(&raw(&["1,"])),
            DefaultCommand::Next(Filter::default().with_words(["1,"])),
        );
    }

    #[test]
    fn leading_comma_demotes_to_word() {
        assert_eq!(
            parse(&raw(&[",1"])),
            DefaultCommand::Next(Filter::default().with_words([",1"])),
        );
    }

    #[test]
    fn whitespace_around_comma_demotes_to_word() {
        assert_eq!(
            parse(&raw(&["1 , 2"])),
            DefaultCommand::Next(Filter::default().with_words(["1 , 2"])),
        );
    }

    #[test]
    fn outer_whitespace_trimmed_then_parsed_as_set() {
        assert_eq!(
            parse(&raw(&["  1,2  "])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Words filter (search terms) ──

    #[test]
    fn bare_word_collected_into_words() {
        assert_eq!(
            parse(&raw(&["hello"])),
            DefaultCommand::Next(Filter::default().with_words(["hello"])),
        );
    }

    #[test]
    fn multiple_bare_words_collected() {
        assert_eq!(
            parse(&raw(&["hello", "world"])),
            DefaultCommand::Next(Filter::default().with_words(["hello", "world"])),
        );
    }

    #[test]
    fn bare_id_with_word_yields_info_with_word_filter() {
        assert_eq!(
            parse(&raw(&["1", "hello"])),
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
            parse(&raw(&["1,2", "hello"])),
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
            parse(&raw(&["hello", "hello"])),
            DefaultCommand::Next(Filter::default().with_words(["hello"])),
        );
    }

    #[test]
    fn surrounding_whitespace_on_word_trimmed() {
        assert_eq!(
            parse(&raw(&["  hello  "])),
            DefaultCommand::Next(Filter::default().with_words(["hello"])),
        );
    }

    #[test]
    fn set_with_zero_segment_demoted_to_word() {
        // Strict SET_RE rejects "0" as an index segment, so the whole token
        // falls through and becomes a single word.
        assert_eq!(
            parse(&raw(&["1,0,2"])),
            DefaultCommand::Next(Filter::default().with_words(["1,0,2"])),
        );
    }

    // ── Empty input ──

    #[test]
    fn empty_input_yields_next_with_empty_filter() {
        assert_eq!(parse(&raw(&[])), DefaultCommand::Next(Filter::default()));
    }
}
