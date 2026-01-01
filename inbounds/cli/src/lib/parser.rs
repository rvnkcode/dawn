use crate::cli::Modification;
use crate::dict;
use dawn::domain::filter::{Filter, IndexRange};
use dawn::domain::task::{Description, Index, TaskModification, UniqueID};
use regex::Regex;
use std::sync::LazyLock;

static INDEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)$").unwrap());
static RANGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-(\d+)$").unwrap());
static UID_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_-]{11}$").unwrap());
static ALPHA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z]{11}$").unwrap());
static ID_SET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+(-\d+)?(,\d+(-\d+)?)*$").unwrap());

enum ParsedItem {
    Index(Index),
    Range(IndexRange),
    UID(UniqueID),
    Word(String),
}

pub fn parse_filter(raw_filters: &[String]) -> Filter {
    let (indices, ranges, uids, words) = parse_items(raw_filters);
    Filter {
        indices,
        ranges,
        uids,
        words,
    }
}

pub fn parse_en_passant_filter(raw_filters: &[String], args: &[String]) -> Filter {
    let items: Vec<ParsedItem> = raw_filters
        .iter()
        .chain(args.iter())
        .flat_map(|chunk| expand_chunk(chunk))
        .map(|fragment| parse_fragment(&fragment))
        .collect();

    let (indices, ranges, uids, words) = partition_items(items);
    Filter {
        indices,
        ranges,
        uids,
        words,
    }
}

fn expand_chunk(chunk: &str) -> Vec<String> {
    let trimmed = chunk.trim();
    if trimmed.is_empty() {
        vec![]
    } else if ID_SET_RE.is_match(trimmed) {
        trimmed.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![trimmed.to_string()]
    }
}

fn parse_items(source: &[String]) -> (Vec<Index>, Vec<IndexRange>, Vec<UniqueID>, Vec<String>) {
    let items: Vec<ParsedItem> = source
        .iter()
        .flat_map(|chunk| expand_chunk(chunk))
        .map(|fragment| parse_fragment(&fragment))
        .collect();
    partition_items(items)
}

fn partition_items(
    items: Vec<ParsedItem>,
) -> (Vec<Index>, Vec<IndexRange>, Vec<UniqueID>, Vec<String>) {
    let mut indices = Vec::new();
    let mut ranges = Vec::new();
    let mut uids = Vec::new();
    let mut words = Vec::new();

    for item in items {
        match item {
            ParsedItem::Index(i) => indices.push(i),
            ParsedItem::Range(r) => ranges.push(r),
            ParsedItem::UID(u) => uids.push(u),
            ParsedItem::Word(w) => words.push(w),
        }
    }
    (indices, ranges, uids, words)
}

fn make_description(words: &[String]) -> anyhow::Result<Option<Description>> {
    if words.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Description::new(&words.join(" "))?))
    }
}

pub fn parse_filter_with_modifications(
    raw_filters: &[String],
    mods: &Modification,
) -> anyhow::Result<(Filter, TaskModification)> {
    let args = &mods.description;

    if raw_filters.is_empty() {
        let (indices, ranges, uids, words) = parse_items(args);
        // Compose description excepts indices, ranges, and uids
        let description = make_description(&words)?;
        let filter = Filter {
            indices,
            ranges,
            uids,
            words: vec![],
        };
        return Ok((filter, TaskModification { description }));
    }

    let (indices, ranges, uids, words) = parse_items(raw_filters);
    let description = make_description(args)?;
    let filter = Filter {
        indices,
        ranges,
        uids,
        words,
    };
    Ok((filter, TaskModification { description }))
}

fn parse_fragment(fragment: &str) -> ParsedItem {
    try_parse_range(fragment)
        .or_else(|| try_parse_index(fragment))
        .or_else(|| try_parse_uid(fragment))
        .unwrap_or_else(|| ParsedItem::Word(fragment.to_string()))
}

fn try_parse_range(fragment: &str) -> Option<ParsedItem> {
    let caps = RANGE_RE.captures(fragment)?;
    let a = caps[1].parse::<usize>().ok()?;
    let b = caps[2].parse::<usize>().ok()?;
    let idx_a = Index::new(a).ok()?;
    let idx_b = Index::new(b).ok()?;

    if idx_a == idx_b {
        return Some(ParsedItem::Index(idx_a));
    }
    Some(ParsedItem::Range(IndexRange::new(idx_a, idx_b).unwrap()))
}

fn try_parse_index(fragment: &str) -> Option<ParsedItem> {
    let caps = INDEX_RE.captures(fragment)?;
    let index = caps[1]
        .parse::<usize>()
        .ok()
        .and_then(|n| Index::new(n).ok())?;
    Some(ParsedItem::Index(index))
}

fn try_parse_uid(fragment: &str) -> Option<ParsedItem> {
    if !UID_RE.is_match(fragment) {
        return None;
    }
    let is_english = ALPHA_RE.is_match(fragment) && dict::is_english_word(&fragment.to_lowercase());
    if is_english {
        return None;
    }
    UniqueID::from_str(fragment).ok().map(ParsedItem::UID)
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

    mod parse_filter_with_modifications_tests {
        use super::*;
        use crate::cli::Modification;
        use dawn::domain::task::Description;

        fn mods(desc: &[&str]) -> Modification {
            Modification {
                description: desc.iter().map(|s| s.to_string()).collect(),
            }
        }

        fn desc(s: &str) -> Option<Description> {
            Some(Description::new(s).unwrap())
        }

        #[test]
        fn empty_filter_extracts_indices_from_args() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&[]), &mods(&["1", "2"])).unwrap();

            assert_eq!(filter.indices.len(), 2);
            assert!(task_mod.description.is_none());
        }

        #[test]
        fn empty_filter_with_words_returns_description() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&[]), &mods(&["hello", "world"])).unwrap();

            assert!(filter.indices.is_empty());
            assert_eq!(task_mod.description, desc("hello world"));
        }

        #[test]
        fn empty_filter_mixed_indices_and_words() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&[]), &mods(&["1", "hello", "2"])).unwrap();

            assert_eq!(filter.indices.len(), 2);
            assert_eq!(task_mod.description, desc("hello"));
        }

        #[test]
        fn with_filter_uses_args_as_description() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&["1", "2"]), &mods(&["new", "description"]))
                    .unwrap();

            assert_eq!(filter.indices.len(), 2);
            assert_eq!(task_mod.description, desc("new description"));
        }

        #[test]
        fn with_filter_empty_args_no_description() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&["1"]), &mods(&[])).unwrap();

            assert_eq!(filter.indices.len(), 1);
            assert!(task_mod.description.is_none());
        }

        #[test]
        fn with_filter_parses_words_in_filter() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&["1", "search"]), &mods(&["new", "desc"]))
                    .unwrap();

            assert_eq!(filter.indices.len(), 1);
            assert_eq!(filter.words, vec!["search"]);
            assert_eq!(task_mod.description, desc("new desc"));
        }

        #[test]
        fn empty_filter_with_range() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&[]), &mods(&["1-5", "update"])).unwrap();

            assert_eq!(filter.ranges.len(), 1);
            assert_eq!(task_mod.description, desc("update"));
        }

        #[test]
        fn empty_filter_with_uid() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&[]), &mods(&["abc12345678", "modify"]))
                    .unwrap();

            assert_eq!(filter.uids.len(), 1);
            assert_eq!(task_mod.description, desc("modify"));
        }

        #[test]
        fn both_empty() {
            let (filter, task_mod) =
                parse_filter_with_modifications(&strs(&[]), &mods(&[])).unwrap();

            assert!(filter.is_empty());
            assert!(task_mod.description.is_none());
        }
    }
}
