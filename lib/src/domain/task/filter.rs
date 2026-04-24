use crate::domain::task::{Index, Status, UniqueID};
use std::collections::HashSet;

#[derive(Debug, Default, PartialEq)]
pub struct Filter {
    uids: HashSet<UniqueID>,
    indices: HashSet<Index>,
    statuses: HashSet<Status>,
    words: Vec<String>,
}

impl Filter {
    pub fn with_uids(mut self, uids: impl IntoIterator<Item = UniqueID>) -> Self {
        self.uids.extend(uids);
        self
    }

    pub fn with_indices(mut self, indices: impl IntoIterator<Item = Index>) -> Self {
        self.indices.extend(indices);
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
            if !trimmed.is_empty() && !self.words.iter().any(|w| w == trimmed) {
                self.words.push(trimmed.to_string());
            }
        }
        self
    }

    pub fn uids(&self) -> &HashSet<UniqueID> {
        &self.uids
    }

    pub fn indices(&self) -> &HashSet<Index> {
        &self.indices
    }

    pub fn statuses(&self) -> &HashSet<Status> {
        &self.statuses
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn is_empty(&self) -> bool {
        self.uids.is_empty()
            && self.indices.is_empty()
            && self.statuses.is_empty()
            && self.words.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(s: &str) -> UniqueID {
        s.parse().expect(s)
    }

    // UIDs

    #[test]
    fn with_uids_single() {
        let filter = Filter::default().with_uids([uid("abcdefghijkl")]);
        assert_eq!(filter.uids().len(), 1);
        assert!(filter.uids().contains(&uid("abcdefghijkl")));
    }

    #[test]
    fn with_uids_multiple() {
        let filter = Filter::default().with_uids([uid("abcdefghijkl"), uid("mnopqrstuvwx")]);
        assert_eq!(filter.uids().len(), 2);
        assert!(filter.uids().contains(&uid("abcdefghijkl")));
        assert!(filter.uids().contains(&uid("mnopqrstuvwx")));
    }

    #[test]
    fn with_uids_deduplicates() {
        let filter = Filter::default().with_uids([uid("abcdefghijkl"), uid("abcdefghijkl")]);
        assert_eq!(filter.uids().len(), 1);
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
        let filter = Filter::default().with_uids([uid("abcdefghijkl")]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_indices_only() {
        let filter = Filter::default().with_indices([Index::new(1).unwrap()]);
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
            .with_uids([uid("abcdefghijkl")])
            .with_statuses([Status::Pending])
            .with_indices([Index::new(1).unwrap()]);
        assert!(!filter.is_empty());
    }

    // with_*()

    #[test]
    fn with_uids_extends() {
        let filter = Filter::default()
            .with_uids([uid("abcdefghijkl")])
            .with_uids([uid("mnopqrstuvwx")]);
        assert_eq!(filter.uids().len(), 2);
        assert!(filter.uids().contains(&uid("abcdefghijkl")));
        assert!(filter.uids().contains(&uid("mnopqrstuvwx")));
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
