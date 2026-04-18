use dawn::domain::task::{Filter, Index, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

static SET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^,\s]+(,[^,\s]+)+$").unwrap());

pub(crate) struct ParsedFilters {
    set: Filter,
    // TODO: info command
    #[allow(dead_code)]
    bare: Filter,
}

impl ParsedFilters {
    pub(crate) fn new(raw_terms: &[String]) -> Self {
        let mut set_uids: HashSet<UniqueID> = HashSet::new();
        let mut set_indices: HashSet<Index> = HashSet::new();
        let mut bare_uids: HashSet<UniqueID> = HashSet::new();
        let mut bare_indices: HashSet<Index> = HashSet::new();

        for fragment in raw_terms {
            let fragment = fragment.trim();

            if SET_RE.is_match(fragment) {
                for seg in fragment.split(',') {
                    try_insert(seg, &mut set_uids, &mut set_indices);
                }
            } else {
                try_insert(fragment, &mut bare_uids, &mut bare_indices);
            }
        }

        Self {
            set: Filter::default()
                .with_uids(set_uids)
                .with_indices(set_indices),
            bare: Filter::default()
                .with_uids(bare_uids)
                .with_indices(bare_indices),
        }
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.set.is_empty() && self.bare.is_empty()
    }

    pub(crate) fn into_set(self) -> Filter {
        self.set
    }
}

fn try_insert(seg: &str, uids: &mut HashSet<UniqueID>, indices: &mut HashSet<Index>) {
    let seg = seg.trim();
    if seg.is_empty() {
        return;
    }
    if let Ok(uid) = UniqueID::from_str(seg) {
        uids.insert(uid);
    } else if let Ok(idx) = Index::from_str(seg) {
        indices.insert(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(terms: &[&str]) -> Vec<String> {
        terms.iter().map(|s| s.to_string()).collect()
    }

    // ── Bare (no comma) ──

    #[test]
    fn parses_single_uid_as_bare() {
        let parsed = ParsedFilters::new(&raw(&["abcdefghijkl"]));
        assert_eq!(parsed.bare.uids().len(), 1);
        assert!(parsed.bare.indices().is_empty());
        assert!(parsed.set.is_empty());
    }

    #[test]
    fn parses_single_index_as_bare() {
        let parsed = ParsedFilters::new(&raw(&["42"]));
        assert_eq!(parsed.bare.indices().len(), 1);
        assert!(parsed.bare.uids().is_empty());
        assert!(parsed.set.is_empty());
    }

    #[test]
    fn parses_12_digit_numeric_as_uid() {
        let parsed = ParsedFilters::new(&raw(&["123456789012"]));
        assert_eq!(parsed.bare.uids().len(), 1);
        assert!(parsed.bare.indices().is_empty());
    }

    // ── Set (comma-separated) ──

    #[test]
    fn parses_comma_separated_indices_as_set() {
        let parsed = ParsedFilters::new(&raw(&["1,2,3"]));
        assert_eq!(parsed.set.indices().len(), 3);
        assert!(parsed.set.uids().is_empty());
        assert!(parsed.bare.is_empty());
    }

    #[test]
    fn parses_mixed_index_and_uid_as_set() {
        let parsed = ParsedFilters::new(&raw(&["1,abcdefghijkl"]));
        assert_eq!(parsed.set.indices().len(), 1);
        assert_eq!(parsed.set.uids().len(), 1);
        assert!(parsed.bare.is_empty());
    }

    // ── Invalid segments silently dropped ──

    #[test]
    fn silently_drops_single_invalid_bare() {
        let parsed = ParsedFilters::new(&raw(&["invalid"]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn silently_drops_zero_bare() {
        let parsed = ParsedFilters::new(&raw(&["0"]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn silently_drops_invalid_segment_in_set() {
        let parsed = ParsedFilters::new(&raw(&["1,invalid,2"]));
        assert_eq!(parsed.set.indices().len(), 2);
        assert!(parsed.bare.is_empty());
    }

    #[test]
    fn silently_drops_all_invalid_set() {
        let parsed = ParsedFilters::new(&raw(&["invalid,xyz"]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn silently_drops_non_ascii() {
        let parsed = ParsedFilters::new(&raw(&["한국어"]));
        assert!(parsed.is_empty());
    }

    // ── Edge cases ──

    #[test]
    fn empty_string_yields_empty() {
        let parsed = ParsedFilters::new(&raw(&[""]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn double_comma_rejected_as_malformed() {
        let parsed = ParsedFilters::new(&raw(&["1,,2"]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn trailing_comma_rejected_as_malformed() {
        let parsed = ParsedFilters::new(&raw(&["1,"]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn leading_comma_rejected_as_malformed() {
        let parsed = ParsedFilters::new(&raw(&[",1"]));
        assert!(parsed.is_empty());
    }

    #[test]
    fn outer_whitespace_trimmed() {
        let parsed = ParsedFilters::new(&raw(&["  1,2  "]));
        assert_eq!(parsed.set.indices().len(), 2);
    }

    #[test]
    fn whitespace_around_comma_rejected() {
        let parsed = ParsedFilters::new(&raw(&["1 , 2"]));
        assert!(parsed.is_empty());
    }

    // ── Multiple args ──

    #[test]
    fn multiple_bare_args_merge_into_bare() {
        let parsed = ParsedFilters::new(&raw(&["1", "2"]));
        assert_eq!(parsed.bare.indices().len(), 2);
        assert!(parsed.set.is_empty());
    }

    #[test]
    fn multiple_set_args_merge_into_set() {
        let parsed = ParsedFilters::new(&raw(&["1,2", "2,3"]));
        // HashSet dedups overlapping "2"
        assert_eq!(parsed.set.indices().len(), 3);
        assert!(parsed.bare.is_empty());
    }

    #[test]
    fn bare_and_set_kept_separate() {
        let parsed = ParsedFilters::new(&raw(&["1", "2,3"]));
        assert_eq!(parsed.bare.indices().len(), 1);
        assert_eq!(parsed.set.indices().len(), 2);
    }

    // ── Dedup ──

    #[test]
    fn duplicates_within_set_are_deduped() {
        let parsed = ParsedFilters::new(&raw(&["1,1,1"]));
        assert_eq!(parsed.set.indices().len(), 1);
    }
}
