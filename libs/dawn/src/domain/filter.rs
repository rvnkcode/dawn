mod index_range;
pub use index_range::{IndexRange, IndexRangeError};

use crate::domain::task::Index;

pub struct Filter {
    pub indices: Vec<Index>,
    pub ranges: Vec<IndexRange>,
}
