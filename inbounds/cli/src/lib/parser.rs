use crate::dict;
use dawn::domain::filter::{Filter, IndexRange};
use dawn::domain::task::{Index, UniqueID};
use regex::Regex;
use std::sync::LazyLock;

static INDEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)$").unwrap());
static RANGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-(\d+)$").unwrap());
static UID_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_-]{11}$").unwrap());
static ALPHA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z]{11}$").unwrap());
static ID_SET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+(-\d+)?(,\d+(-\d+)?)*$").unwrap());

enum ParsedFilter {
    Index(Index),
    Range(IndexRange),
    UID(UniqueID),
}

pub fn parse_filter(raw_filters: &[String]) -> Filter {
    let mut filter = Filter::default();
    parse_chunks(raw_filters, &mut filter);
    filter
}

pub fn parse_en_passant_filter(raw_filters: &[String], args: &[String]) -> Filter {
    let mut filter = Filter::default();
    parse_chunks(raw_filters, &mut filter);
    parse_chunks(args, &mut filter);
    filter
}

fn parse_chunks(chunks: &[String], filter: &mut Filter) {
    for chunk in chunks {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Only split on comma for pure ID set patterns (e.g., "1,2,3" or "1-5,7")
        if ID_SET_RE.is_match(trimmed) {
            for fragment in trimmed.split(',') {
                process_fragment(fragment.trim(), filter);
            }
        } else {
            process_fragment(trimmed, filter);
        }
    }
}

fn process_fragment(fragment: &str, filter: &mut Filter) {
    if fragment.is_empty() {
        return;
    }
    match parse_fragment(fragment) {
        Some(ParsedFilter::Index(idx)) => filter.indices.push(idx),
        Some(ParsedFilter::Range(range)) => filter.ranges.push(range),
        Some(ParsedFilter::UID(uid)) => filter.uids.push(uid),
        None => filter.words.push(fragment.to_string()),
    }
}

fn parse_fragment(fragment: &str) -> Option<ParsedFilter> {
    // Parse index range (e.g. "3-7")
    if let Some(caps) = RANGE_RE.captures(fragment) {
        let a = caps[1].parse::<usize>().ok()?;
        let b = caps[2].parse::<usize>().ok()?;
        let idx_a = Index::new(a).ok()?;
        let idx_b = Index::new(b).ok()?;

        return match IndexRange::new(idx_a, idx_b) {
            Ok(range) => Some(ParsedFilter::Range(range)),
            Err(_) => Some(ParsedFilter::Index(idx_a)),
        };
    }
    // Parse single index (e.g. "5")
    if let Some(caps) = INDEX_RE.captures(fragment) {
        let n = caps[1].parse::<usize>().ok()?;
        let index = Index::new(n).ok()?;
        return Some(ParsedFilter::Index(index));
    }
    // UID parsing (11 characters, alphanumeric + _-)
    if !UID_RE.is_match(fragment) {
        return None;
    }
    // Pure alphabetic in dictionary -> not a UID
    if ALPHA_RE.is_match(fragment) {
        let lower_fragment = fragment.to_lowercase();
        if dict::is_english_word(&lower_fragment) {
            return None;
        }
    }
    UniqueID::from_str(fragment).ok().map(ParsedFilter::UID)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::strs;

    #[test]
    fn parse_single_index() {
        let filter = parse_filter(&strs(&["1"]));

        assert_eq!(filter.indices.len(), 1);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert!(filter.ranges.is_empty());
    }

    #[test]
    fn parse_comma_separated_indices() {
        let filter = parse_filter(&strs(&["1,2,3"]));

        assert_eq!(filter.indices.len(), 3);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
        assert_eq!(filter.indices[2], Index::new(3).unwrap());
    }

    #[test]
    fn parse_range() {
        let filter = parse_filter(&strs(&["1-5"]));

        assert!(filter.indices.is_empty());
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.ranges[0].start(), &Index::new(1).unwrap());
        assert_eq!(filter.ranges[0].end(), &Index::new(5).unwrap());
    }

    #[test]
    fn parse_reversed_range_normalizes() {
        let filter = parse_filter(&strs(&["5-1"]));

        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.ranges[0].start(), &Index::new(1).unwrap());
        assert_eq!(filter.ranges[0].end(), &Index::new(5).unwrap());
    }

    #[test]
    fn parse_same_index_range_becomes_index() {
        let filter = parse_filter(&strs(&["3-3"]));

        assert_eq!(filter.indices.len(), 1);
        assert_eq!(filter.indices[0], Index::new(3).unwrap());
        assert!(filter.ranges.is_empty());
    }

    #[test]
    fn parse_mixed_indices_and_ranges() {
        let filter = parse_filter(&strs(&["1,3-5,7"]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(7).unwrap());
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.ranges[0].start(), &Index::new(3).unwrap());
        assert_eq!(filter.ranges[0].end(), &Index::new(5).unwrap());
    }

    #[test]
    fn parse_invalid_input_goes_to_words() {
        let filter = parse_filter(&strs(&["abc", "0", "-1"]));

        assert!(filter.indices.is_empty());
        assert!(filter.ranges.is_empty());
        assert_eq!(filter.words, vec!["abc", "0", "-1"]);
    }

    #[test]
    fn parse_multiple_raw_filters() {
        let filter = parse_filter(&strs(&["1,2", "3-5"]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.ranges.len(), 1);
    }

    #[test]
    fn parse_whitespace_trimmed() {
        // ID set without internal spaces
        let filter = parse_filter(&strs(&[" 1,2 "]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
    }

    #[test]
    fn parse_spaces_around_comma_becomes_word() {
        // "1 , 2" with spaces around comma is NOT a valid ID set
        let filter = parse_filter(&strs(&["1 , 2"]));

        assert!(filter.indices.is_empty());
        assert_eq!(filter.words, vec!["1 , 2"]);
    }

    #[test]
    fn parse_uid_with_numbers() {
        let filter = parse_filter(&strs(&["abc12345678"]));
        assert_eq!(filter.uids.len(), 1);
    }

    #[test]
    fn parse_uid_with_underscore() {
        let filter = parse_filter(&strs(&["abcdefghij_"]));
        assert_eq!(filter.uids.len(), 1);
    }

    #[test]
    fn parse_uid_with_hyphen() {
        let filter = parse_filter(&strs(&["abcdefghij-"]));
        assert_eq!(filter.uids.len(), 1);
    }

    #[test]
    fn parse_english_word_goes_to_words() {
        let filter = parse_filter(&strs(&["information"]));

        assert!(filter.uids.is_empty());
        assert_eq!(filter.words, vec!["information"]);
    }

    #[test]
    fn parse_non_word_alphabetic_as_uid() {
        let filter = parse_filter(&strs(&["abcdefghijk"]));

        assert_eq!(filter.uids.len(), 1);
    }

    #[test]
    fn parse_mixed_indices_ranges_and_uids() {
        // Mixed types as separate arguments (not comma-joined)
        let filter = parse_filter(&strs(&["1", "abc12345678", "3-5"]));

        assert_eq!(filter.indices.len(), 1);
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.uids.len(), 1);
    }

    #[test]
    fn parse_mixed_id_and_uid_comma_becomes_word() {
        // "1,abc12345678,3-5" is NOT a pure ID set → becomes word
        let filter = parse_filter(&strs(&["1,abc12345678,3-5"]));

        assert!(filter.indices.is_empty());
        assert!(filter.ranges.is_empty());
        assert!(filter.uids.is_empty());
        assert_eq!(filter.words, vec!["1,abc12345678,3-5"]);
    }

    #[test]
    fn parse_en_passant_combines_filters_and_args() {
        let filter = parse_en_passant_filter(&strs(&["1,2"]), &strs(&["3,4"]));

        assert_eq!(filter.indices.len(), 4);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
        assert_eq!(filter.indices[2], Index::new(3).unwrap());
        assert_eq!(filter.indices[3], Index::new(4).unwrap());
    }

    #[test]
    fn parse_en_passant_with_empty_filters() {
        let filter = parse_en_passant_filter(&strs(&[]), &strs(&["1,2"]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
    }

    #[test]
    fn parse_en_passant_with_empty_args() {
        let filter = parse_en_passant_filter(&strs(&["1,2"]), &strs(&[]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
    }

    #[test]
    fn parse_en_passant_with_mixed_types() {
        // Separate arguments for different types
        let filter = parse_en_passant_filter(&strs(&["1", "abc12345678"]), &strs(&["3-5", "word"]));

        assert_eq!(filter.indices.len(), 1);
        assert_eq!(filter.uids.len(), 1);
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.words, vec!["word"]);
    }

    #[test]
    fn parse_en_passant_mixed_comma_becomes_word() {
        // "1,abc12345678" is not a pure ID set
        let filter = parse_en_passant_filter(&strs(&["1,abc12345678"]), &strs(&["3-5"]));

        assert!(filter.indices.is_empty());
        assert!(filter.uids.is_empty());
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.words, vec!["1,abc12345678"]);
    }

    #[test]
    fn parse_en_passant_both_empty() {
        let filter = parse_en_passant_filter(&strs(&[]), &strs(&[]));

        assert!(filter.is_empty());
    }

    #[test]
    fn parse_comma_in_non_id_preserved() {
        let filter = parse_filter(&strs(&["foo,bar"]));

        assert!(filter.indices.is_empty());
        assert_eq!(filter.words, vec!["foo,bar"]);
    }

    #[test]
    fn parse_mixed_id_and_word_not_split() {
        // "1,foo" is NOT a pure ID set → becomes word
        let filter = parse_filter(&strs(&["1,foo"]));

        assert!(filter.indices.is_empty());
        assert_eq!(filter.words, vec!["1,foo"]);
    }

    #[test]
    fn parse_uid_comma_id_not_split() {
        // "abc12345678,1" is NOT a pure ID set → becomes word
        let filter = parse_filter(&strs(&["abc12345678,1"]));

        assert!(filter.uids.is_empty());
        assert!(filter.indices.is_empty());
        assert_eq!(filter.words, vec!["abc12345678,1"]);
    }
}
