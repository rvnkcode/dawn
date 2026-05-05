use std::collections::HashSet;

use crate::domain::task::{Index, IndexRange, Status, UuidPrefix};

#[derive(Debug, Default, PartialEq)]
pub struct Filter {
    uuids: HashSet<UuidPrefix>,
    indices: HashSet<Index>,
    index_ranges: HashSet<IndexRange>,
    statuses: HashSet<Status>,
    words: Vec<String>,
}

impl Filter {
    pub fn with_uuids(mut self, uuids: impl IntoIterator<Item = UuidPrefix>) -> Self {
        self.uuids.extend(uuids);
        self
    }

    pub fn with_indices(mut self, indices: impl IntoIterator<Item = Index>) -> Self {
        self.indices.extend(indices);
        self
    }

    pub fn with_index_ranges(mut self, index_ranges: impl IntoIterator<Item = IndexRange>) -> Self {
        self.index_ranges.extend(index_ranges);
        self
    }

    pub fn with_statuses(mut self, statuses: impl IntoIterator<Item = Status>) -> Self {
        self.statuses.extend(statuses);
        self
    }

    pub fn with_words(mut self, words: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for word in words {
            let owned: String = word.into();
            let trimmed = owned.trim();
            // de-duplicate
            if !trimmed.is_empty() && !self.words.iter().any(|w| w == trimmed) {
                self.words.push(trimmed.to_string());
            }
        }
        self
    }

    pub fn uuids(&self) -> &HashSet<UuidPrefix> {
        &self.uuids
    }

    pub fn indices(&self) -> &HashSet<Index> {
        &self.indices
    }

    pub fn index_ranges(&self) -> &HashSet<IndexRange> {
        &self.index_ranges
    }

    pub fn statuses(&self) -> &HashSet<Status> {
        &self.statuses
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn is_empty(&self) -> bool {
        self.uuids.is_empty()
            && self.indices.is_empty()
            && self.index_ranges.is_empty()
            && self.statuses.is_empty()
            && self.words.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(n: u128) -> UuidPrefix {
        UuidPrefix::from(uuid::Uuid::from_u128(n))
    }

    fn range(from: usize, to: usize) -> IndexRange {
        IndexRange::new(Index::new(from).unwrap(), Index::new(to).unwrap()).unwrap()
    }

    // UIDs

    #[test]
    fn with_uids_single() {
        let filter = Filter::default().with_uuids([uuid(1)]);
        assert_eq!(filter.uuids().len(), 1);
        assert!(filter.uuids().contains(&uuid(1)));
    }

    #[test]
    fn with_uids_multiple() {
        let filter = Filter::default().with_uuids([uuid(1), uuid(2)]);
        assert_eq!(filter.uuids().len(), 2);
        assert!(filter.uuids().contains(&uuid(1)));
        assert!(filter.uuids().contains(&uuid(2)));
    }

    #[test]
    fn with_uids_deduplicates() {
        let filter = Filter::default().with_uuids([uuid(1), uuid(1)]);
        assert_eq!(filter.uuids().len(), 1);
    }

    // Indices

    #[test]
    fn with_indices_single() {
        let filter = Filter::default().with_indices([Index::new(1).unwrap()]);
        assert_eq!(filter.indices().len(), 1);
        assert!(filter.indices().contains(&Index::new(1).unwrap()));
    }

    #[test]
    fn with_indices_multiple() {
        let filter =
            Filter::default().with_indices([Index::new(1).unwrap(), Index::new(2).unwrap()]);
        assert_eq!(filter.indices().len(), 2);
        assert!(filter.indices().contains(&Index::new(1).unwrap()));
        assert!(filter.indices().contains(&Index::new(2).unwrap()));
    }

    #[test]
    fn with_indices_deduplicates() {
        let filter =
            Filter::default().with_indices([Index::new(1).unwrap(), Index::new(1).unwrap()]);
        assert_eq!(filter.indices().len(), 1);
    }

    // Index ranges

    #[test]
    fn with_index_ranges_single() {
        let filter = Filter::default().with_index_ranges([range(1, 3)]);
        assert_eq!(filter.index_ranges().len(), 1);
        assert!(filter.index_ranges().contains(&range(1, 3)));
    }

    #[test]
    fn with_index_ranges_multiple() {
        let filter = Filter::default().with_index_ranges([range(1, 3), range(5, 7)]);
        assert_eq!(filter.index_ranges().len(), 2);
        assert!(filter.index_ranges().contains(&range(1, 3)));
        assert!(filter.index_ranges().contains(&range(5, 7)));
    }

    #[test]
    fn with_index_ranges_deduplicates() {
        let filter = Filter::default().with_index_ranges([range(1, 3), range(1, 3)]);
        assert_eq!(filter.index_ranges().len(), 1);
    }

    #[test]
    fn with_index_ranges_dedup_after_swap_normalization() {
        let filter = Filter::default().with_index_ranges([range(1, 3), range(3, 1)]);
        assert_eq!(filter.index_ranges().len(), 1);
    }

    // Statuses

    #[test]
    fn with_statuses_single() {
        let filter = Filter::default().with_statuses([Status::Pending]);
        assert_eq!(filter.statuses().len(), 1);
        assert!(filter.statuses().contains(&Status::Pending));
    }

    #[test]
    fn with_statuses_multiple() {
        let filter = Filter::default().with_statuses([Status::Pending, Status::Completed]);
        assert_eq!(filter.statuses().len(), 2);
        assert!(filter.statuses().contains(&Status::Pending));
        assert!(filter.statuses().contains(&Status::Completed));
    }

    #[test]
    fn with_statuses_deduplicates() {
        let filter = Filter::default().with_statuses([Status::Pending, Status::Pending]);
        assert_eq!(filter.statuses().len(), 1);
    }

    // is_empty()

    #[test]
    fn is_empty_with_uids_only() {
        let filter = Filter::default().with_uuids([uuid(1)]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_indices_only() {
        let filter = Filter::default().with_indices([Index::new(1).unwrap()]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_index_ranges_only() {
        let filter = Filter::default().with_index_ranges([range(1, 3)]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_statuses_only() {
        let filter = Filter::default().with_statuses([Status::Pending]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_words_only() {
        let filter = Filter::default().with_words(["example"]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_all() {
        let filter = Filter::default()
            .with_uuids([uuid(1)])
            .with_statuses([Status::Pending])
            .with_indices([Index::new(1).unwrap()]);
        assert!(!filter.is_empty());
    }

    // with_*()

    #[test]
    fn with_uids_extends() {
        let filter = Filter::default()
            .with_uuids([uuid(1)])
            .with_uuids([uuid(2)]);
        assert_eq!(filter.uuids().len(), 2);
        assert!(filter.uuids().contains(&uuid(1)));
        assert!(filter.uuids().contains(&uuid(2)));
    }

    #[test]
    fn with_indices_extends() {
        let filter = Filter::default()
            .with_indices([Index::new(1).unwrap()])
            .with_indices([Index::new(2).unwrap()]);
        assert_eq!(filter.indices().len(), 2);
        assert!(filter.indices().contains(&Index::new(1).unwrap()));
        assert!(filter.indices().contains(&Index::new(2).unwrap()));
    }

    #[test]
    fn with_index_ranges_extends() {
        let filter = Filter::default()
            .with_index_ranges([range(1, 3)])
            .with_index_ranges([range(5, 7)]);
        assert_eq!(filter.index_ranges().len(), 2);
        assert!(filter.index_ranges().contains(&range(1, 3)));
        assert!(filter.index_ranges().contains(&range(5, 7)));
    }

    #[test]
    fn with_statuses_extends() {
        let filter = Filter::default()
            .with_statuses([Status::Pending])
            .with_statuses([Status::Completed]);
        assert_eq!(filter.statuses().len(), 2);
        assert!(filter.statuses().contains(&Status::Pending));
        assert!(filter.statuses().contains(&Status::Completed));
    }

    #[test]
    fn with_words_extends() {
        let filter = Filter::default()
            .with_words(["example"])
            .with_words(["test"]);

        assert_eq!(filter.words().len(), 2);
        assert!(filter.words().contains(&"example".to_string()));
        assert!(filter.words().contains(&"test".to_string()));
    }

    #[test]
    fn with_words_trims_whitespace() {
        let filter = Filter::default().with_words(["  hello  ", "\tworld\n"]);

        assert_eq!(filter.words(), &["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn with_words_skips_empty_and_whitespace_only() {
        let filter = Filter::default().with_words(["", "   ", "\t\n", "valid"]);

        assert_eq!(filter.words(), &["valid".to_string()]);
    }

    #[test]
    fn with_words_deduplicates() {
        let filter = Filter::default().with_words(["foo", "bar", "foo"]);

        assert_eq!(filter.words(), &["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn with_words_deduplicates_across_calls() {
        let filter = Filter::default()
            .with_words(["foo", "bar"])
            .with_words(["bar", "baz"]);

        assert_eq!(
            filter.words(),
            &["foo".to_string(), "bar".to_string(), "baz".to_string()],
        );
    }

    #[test]
    fn with_words_deduplicates_after_trimming() {
        let filter = Filter::default().with_words(["foo", "  foo  "]);

        assert_eq!(filter.words(), &["foo".to_string()]);
    }
}
