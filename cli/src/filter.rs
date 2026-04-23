use dawn::domain::task::{Filter, Index, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

static SET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^,\s]+(,[^,\s]+)+$").unwrap());

#[derive(Debug, PartialEq)]
pub(crate) enum DefaultCommand {
    Next(Filter),
    Info(Filter),
}

pub(crate) fn parse(raw_terms: &[String]) -> DefaultCommand {
    let mut uids = HashSet::new();
    let mut indices = HashSet::new();
    let mut has_bare_id = false;

    for raw in raw_terms {
        let fragment = raw.trim();

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
        }
    }

    let filter = Filter::default().with_uids(uids).with_indices(indices);
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

    // ── Invalid bare → dropped, does not flip to Info ──

    #[test]
    fn invalid_bare_drops_silently_and_stays_next() {
        assert_eq!(
            parse(&raw(&["invalid"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    #[test]
    fn zero_bare_drops_silently_and_stays_next() {
        assert_eq!(parse(&raw(&["0"])), DefaultCommand::Next(Filter::default()),);
    }

    #[test]
    fn non_ascii_bare_drops_silently_and_stays_next() {
        assert_eq!(
            parse(&raw(&["한국어"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    #[test]
    fn invalid_bare_mixed_with_set_stays_next() {
        assert_eq!(
            parse(&raw(&["invalid", "1,2"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Invalid segment inside a set → dropped, set stays Next ──

    #[test]
    fn invalid_segment_in_set_drops_but_keeps_valid() {
        assert_eq!(
            parse(&raw(&["1,invalid,2"])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    #[test]
    fn all_invalid_set_yields_next_with_empty_filter() {
        assert_eq!(
            parse(&raw(&["invalid,xyz"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    // ── Malformed tokens (comma shapes not matching SET_RE) ──
    //
    // These fall through to the bare branch; UniqueID/Index parsing then fails
    // on the literal string (which contains commas), so they drop without
    // flipping has_bare_id.

    #[test]
    fn empty_string_yields_next_with_empty_filter() {
        assert_eq!(parse(&raw(&[""])), DefaultCommand::Next(Filter::default()),);
    }

    #[test]
    fn double_comma_rejected_as_malformed() {
        assert_eq!(
            parse(&raw(&["1,,2"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    #[test]
    fn trailing_comma_rejected_as_malformed() {
        assert_eq!(
            parse(&raw(&["1,"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    #[test]
    fn leading_comma_rejected_as_malformed() {
        assert_eq!(
            parse(&raw(&[",1"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    #[test]
    fn whitespace_around_comma_rejected() {
        assert_eq!(
            parse(&raw(&["1 , 2"])),
            DefaultCommand::Next(Filter::default()),
        );
    }

    #[test]
    fn outer_whitespace_trimmed_then_parsed_as_set() {
        assert_eq!(
            parse(&raw(&["  1,2  "])),
            DefaultCommand::Next(Filter::default().with_indices([idx(1), idx(2)])),
        );
    }

    // ── Empty input ──

    #[test]
    fn empty_input_yields_next_with_empty_filter() {
        assert_eq!(parse(&raw(&[])), DefaultCommand::Next(Filter::default()));
    }
}
