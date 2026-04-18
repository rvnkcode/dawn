use dawn::domain::task::{Filter, Index, UniqueID};
use regex::Regex;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::LazyLock;

static SET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^,\s]+(,[^,\s]+)+$").unwrap());

pub(crate) struct ParsedFilters {
    set_uids: HashSet<UniqueID>,
    set_indices: HashSet<Index>,
    // TODO: info command / next / all
    #[allow(dead_code)]
    bare_uids: HashSet<UniqueID>,
    // TODO: info command / next / all
    #[allow(dead_code)]
    bare_indices: HashSet<Index>,
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
            set_uids,
            set_indices,
            bare_uids,
            bare_indices,
        }
    }

    pub(crate) fn into_set(self) -> Filter {
        Filter::default()
            .with_uids(self.set_uids)
            .with_indices(self.set_indices)
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.set_is_empty() && self.bare_is_empty()
    }

    #[cfg(test)]
    fn set_is_empty(&self) -> bool {
        self.set_uids.is_empty() && self.set_indices.is_empty()
    }

    #[cfg(test)]
    fn bare_is_empty(&self) -> bool {
        self.bare_uids.is_empty() && self.bare_indices.is_empty()
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
        assert_eq!(parsed.bare_uids.len(), 1);
        assert!(parsed.bare_indices.is_empty());
        assert!(parsed.set_is_empty());
    }

    #[test]
    fn parses_single_index_as_bare() {
        let parsed = ParsedFilters::new(&raw(&["42"]));
        assert_eq!(parsed.bare_indices.len(), 1);
        assert!(parsed.bare_uids.is_empty());
        assert!(parsed.set_is_empty());
    }

    #[test]
    fn parses_12_digit_numeric_as_uid() {
        let parsed = ParsedFilters::new(&raw(&["123456789012"]));
        assert_eq!(parsed.bare_uids.len(), 1);
        assert!(parsed.bare_indices.is_empty());
    }

    // ── Set (comma-separated) ──

    #[test]
    fn parses_comma_separated_indices_as_set() {
        let parsed = ParsedFilters::new(&raw(&["1,2,3"]));
        assert_eq!(parsed.set_indices.len(), 3);
        assert!(parsed.set_uids.is_empty());
        assert!(parsed.bare_is_empty());
    }

    #[test]
    fn parses_mixed_index_and_uid_as_set() {
        let parsed = ParsedFilters::new(&raw(&["1,abcdefghijkl"]));
        assert_eq!(parsed.set_indices.len(), 1);
        assert_eq!(parsed.set_uids.len(), 1);
        assert!(parsed.bare_is_empty());
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
        assert_eq!(parsed.set_indices.len(), 2);
        assert!(parsed.bare_is_empty());
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
        assert_eq!(parsed.set_indices.len(), 2);
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
        assert_eq!(parsed.bare_indices.len(), 2);
        assert!(parsed.set_is_empty());
    }

    #[test]
    fn multiple_set_args_merge_into_set() {
        let parsed = ParsedFilters::new(&raw(&["1,2", "2,3"]));
        // HashSet dedups overlapping "2"
        assert_eq!(parsed.set_indices.len(), 3);
        assert!(parsed.bare_is_empty());
    }

    #[test]
    fn bare_and_set_kept_separate() {
        let parsed = ParsedFilters::new(&raw(&["1", "2,3"]));
        assert_eq!(parsed.bare_indices.len(), 1);
        assert_eq!(parsed.set_indices.len(), 2);
    }

    // ── Dedup ──

    #[test]
    fn duplicates_within_set_are_deduped() {
        let parsed = ParsedFilters::new(&raw(&["1,1,1"]));
        assert_eq!(parsed.set_indices.len(), 1);
    }

    // ── into_set() ──

    #[test]
    fn into_set_builds_filter_from_set_terms() {
        let filter = ParsedFilters::new(&raw(&["1,2,abcdefghijkl"])).into_set();
        assert_eq!(filter.indices().len(), 2);
        assert_eq!(filter.uids().len(), 1);
    }

    #[test]
    fn into_set_drops_bare_terms() {
        let filter = ParsedFilters::new(&raw(&["1", "abcdefghijkl"])).into_set();
        assert!(filter.is_empty());
    }
}
