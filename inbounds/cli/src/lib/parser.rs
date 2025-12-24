use dawn::domain::filter::{Filter, IndexRange};
use dawn::domain::task::Index;
use regex::Regex;
use std::sync::LazyLock;

static INDEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)$").unwrap());
static RANGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-(\d+)$").unwrap());

enum ParsedFilter {
    Index(Index),
    Range(IndexRange),
}

fn parse_fragment(fragment: &str) -> Option<ParsedFilter> {
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

    if let Some(caps) = INDEX_RE.captures(fragment) {
        let n = caps[1].parse::<usize>().ok()?;
        let index = Index::new(n).ok()?;
        return Some(ParsedFilter::Index(index));
    }

    None
}

pub struct Parser;

impl Parser {
    pub fn parse_filter(raw_filters: &[String]) -> Filter {
        let mut indices: Vec<Index> = Vec::new();
        let mut ranges: Vec<IndexRange> = Vec::new();

        for chunk in raw_filters {
            for fragment in chunk.split(',') {
                match parse_fragment(fragment.trim()) {
                    Some(ParsedFilter::Index(idx)) => indices.push(idx),
                    Some(ParsedFilter::Range(range)) => ranges.push(range),
                    None => {}
                }
            }
        }

        Filter { indices, ranges }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::strs;

    #[test]
    fn parse_single_index() {
        let filter = Parser::parse_filter(&strs(&["1"]));

        assert_eq!(filter.indices.len(), 1);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert!(filter.ranges.is_empty());
    }

    #[test]
    fn parse_comma_separated_indices() {
        let filter = Parser::parse_filter(&strs(&["1,2,3"]));

        assert_eq!(filter.indices.len(), 3);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
        assert_eq!(filter.indices[2], Index::new(3).unwrap());
    }

    #[test]
    fn parse_range() {
        let filter = Parser::parse_filter(&strs(&["1-5"]));

        assert!(filter.indices.is_empty());
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.ranges[0].start(), &Index::new(1).unwrap());
        assert_eq!(filter.ranges[0].end(), &Index::new(5).unwrap());
    }

    #[test]
    fn parse_reversed_range_normalizes() {
        let filter = Parser::parse_filter(&strs(&["5-1"]));

        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.ranges[0].start(), &Index::new(1).unwrap());
        assert_eq!(filter.ranges[0].end(), &Index::new(5).unwrap());
    }

    #[test]
    fn parse_same_index_range_becomes_index() {
        let filter = Parser::parse_filter(&strs(&["3-3"]));

        assert_eq!(filter.indices.len(), 1);
        assert_eq!(filter.indices[0], Index::new(3).unwrap());
        assert!(filter.ranges.is_empty());
    }

    #[test]
    fn parse_mixed_indices_and_ranges() {
        let filter = Parser::parse_filter(&strs(&["1,3-5,7"]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(7).unwrap());
        assert_eq!(filter.ranges.len(), 1);
        assert_eq!(filter.ranges[0].start(), &Index::new(3).unwrap());
        assert_eq!(filter.ranges[0].end(), &Index::new(5).unwrap());
    }

    #[test]
    fn parse_invalid_input_ignored() {
        let filter = Parser::parse_filter(&strs(&["abc", "0", "-1"]));

        assert!(filter.indices.is_empty());
        assert!(filter.ranges.is_empty());
    }

    #[test]
    fn parse_multiple_raw_filters() {
        let filter = Parser::parse_filter(&strs(&["1,2", "3-5"]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.ranges.len(), 1);
    }

    #[test]
    fn parse_whitespace_trimmed() {
        let filter = Parser::parse_filter(&strs(&[" 1 , 2 "]));

        assert_eq!(filter.indices.len(), 2);
        assert_eq!(filter.indices[0], Index::new(1).unwrap());
        assert_eq!(filter.indices[1], Index::new(2).unwrap());
    }
}
