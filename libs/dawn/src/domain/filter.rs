mod index_range;

use crate::domain::task::{Index, UniqueID};
pub use index_range::{IndexRange, IndexRangeError};

#[derive(Default)]
pub struct Filter {
    pub indices: Vec<Index>,
    pub ranges: Vec<IndexRange>,
    pub uids: Vec<UniqueID>,
    pub words: Vec<String>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
            && self.ranges.is_empty()
            && self.uids.is_empty()
            && self.words.is_empty()
    }
}
