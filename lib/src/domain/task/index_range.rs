use crate::domain::task::Index;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct IndexRange {
    start: Index,
    end: Index,
}

impl IndexRange {
    pub fn new(start: Index, end: Index) -> Result<Self, Index> {
        let (low, high) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        if low == high {
            Err(low) // Fallback to the Index
        } else {
            Ok(Self {
                start: low,
                end: high,
            })
        }
    }

    pub fn start(&self) -> &Index {
        &self.start
    }

    pub fn end(&self) -> &Index {
        &self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(n: usize) -> Index {
        Index::new(n).unwrap()
    }

    #[test]
    fn new_keeps_order_when_start_lt_end() {
        let range = IndexRange::new(idx(1), idx(3)).unwrap();
        assert_eq!(range.start(), &idx(1));
        assert_eq!(range.end(), &idx(3));
    }

    #[test]
    fn new_swaps_when_start_gt_end() {
        let range = IndexRange::new(idx(5), idx(3)).unwrap();
        assert_eq!(range.start(), &idx(3));
        assert_eq!(range.end(), &idx(5));
    }

    #[test]
    fn new_returns_err_with_index_when_start_eq_end() {
        let err = IndexRange::new(idx(2), idx(2)).unwrap_err();
        assert_eq!(err, idx(2));
    }
}
