use crate::domain::task::Index;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct IndexRange {
    from: Index,
    to: Index,
}

impl IndexRange {
    pub fn new(from: Index, to: Index) -> Result<Self, Index> {
        let (low, high) = if from <= to { (from, to) } else { (to, from) };
        if low == high {
            Err(low)
        } else {
            Ok(Self {
                from: low,
                to: high,
            })
        }
    }

    pub fn from(&self) -> &Index {
        &self.from
    }

    pub fn to(&self) -> &Index {
        &self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(n: usize) -> Index {
        Index::new(n).unwrap()
    }

    #[test]
    fn new_keeps_order_when_from_lt_to() {
        let range = IndexRange::new(idx(1), idx(3)).unwrap();
        assert_eq!(range.from(), &idx(1));
        assert_eq!(range.to(), &idx(3));
    }

    #[test]
    fn new_swaps_when_from_gt_to() {
        let range = IndexRange::new(idx(5), idx(3)).unwrap();
        assert_eq!(range.from(), &idx(3));
        assert_eq!(range.to(), &idx(5));
    }

    #[test]
    fn new_returns_err_with_index_when_from_eq_to() {
        let err = IndexRange::new(idx(2), idx(2)).unwrap_err();
        assert_eq!(err, idx(2));
    }
}
