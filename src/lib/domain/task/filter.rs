use crate::domain::task::{Status, UniqueID};
use std::collections::HashSet;

#[derive(Default)]
pub struct Filter {
    uids: HashSet<UniqueID>,
    statuses: HashSet<Status>,
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_uids(self, uids: impl IntoIterator<Item = UniqueID>) -> Self {
        Self {
            uids: uids.into_iter().collect(),
            ..self
        }
    }

    pub fn with_statuses(self, statuses: impl IntoIterator<Item = Status>) -> Self {
        Self {
            statuses: statuses.into_iter().collect(),
            ..self
        }
    }

    pub fn uids(&self) -> &HashSet<UniqueID> {
        &self.uids
    }

    pub fn statuses(&self) -> &HashSet<Status> {
        &self.statuses
    }

    pub fn is_empty(&self) -> bool {
        self.uids.is_empty() && self.statuses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(s: &str) -> UniqueID {
        s.parse().expect(s)
    }

    #[test]
    fn new_is_empty() {
        assert!(Filter::new().is_empty());
    }

    // UIDs

    #[test]
    fn with_uids_single() {
        let filter = Filter::new().with_uids([uid("abcdefghijkl")]);
        assert_eq!(filter.uids().len(), 1);
        assert!(filter.uids().contains(&uid("abcdefghijkl")));
    }

    #[test]
    fn with_uids_multiple() {
        let filter = Filter::new().with_uids([uid("abcdefghijkl"), uid("mnopqrstuvwx")]);
        assert_eq!(filter.uids().len(), 2);
        assert!(filter.uids().contains(&uid("abcdefghijkl")));
        assert!(filter.uids().contains(&uid("mnopqrstuvwx")));
    }

    #[test]
    fn with_uids_deduplicates() {
        let filter = Filter::new().with_uids([uid("abcdefghijkl"), uid("abcdefghijkl")]);
        assert_eq!(filter.uids().len(), 1);
    }

    // Statuses

    #[test]
    fn with_statuses_single() {
        let filter = Filter::new().with_statuses([Status::Pending]);
        assert_eq!(filter.statuses().len(), 1);
        assert!(filter.statuses().contains(&Status::Pending));
    }

    #[test]
    fn with_statuses_multiple() {
        let filter = Filter::new().with_statuses([Status::Pending, Status::Completed]);
        assert_eq!(filter.statuses().len(), 2);
        assert!(filter.statuses().contains(&Status::Pending));
        assert!(filter.statuses().contains(&Status::Completed));
    }

    #[test]
    fn with_statuses_deduplicates() {
        let filter = Filter::new().with_statuses([Status::Pending, Status::Pending]);
        assert_eq!(filter.statuses().len(), 1);
    }

    // is_empty()

    #[test]
    fn is_empty_with_uids_only() {
        let filter = Filter::new().with_uids([uid("abcdefghijkl")]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_statuses_only() {
        let filter = Filter::new().with_statuses([Status::Pending]);
        assert!(!filter.is_empty());
    }

    #[test]
    fn is_empty_with_both() {
        let filter = Filter::new()
            .with_uids([uid("abcdefghijkl")])
            .with_statuses([Status::Pending]);
        assert!(!filter.is_empty());
    }

    // with_*()

    #[test]
    fn with_uids_last_call_wins() {
        let filter = Filter::new()
            .with_uids([uid("abcdefghijkl")])
            .with_uids([uid("mnopqrstuvwx")]);
        assert_eq!(filter.uids().len(), 1);
        assert!(filter.uids().contains(&uid("mnopqrstuvwx")));
    }

    #[test]
    fn with_statuses_last_call_wins() {
        let filter = Filter::new()
            .with_statuses([Status::Pending])
            .with_statuses([Status::Completed]);
        assert_eq!(filter.statuses().len(), 1);
        assert!(filter.statuses().contains(&Status::Completed));
    }
}
