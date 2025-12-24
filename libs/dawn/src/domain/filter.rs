mod index_range;
pub use index_range::{IndexRange, IndexRangeError};

use crate::domain::task::{Index, UniqueID};

pub struct Filter {
    pub indices: Vec<Index>,
    pub ranges: Vec<IndexRange>,
    pub uids: Vec<UniqueID>,
    pub words: Vec<String>,
}
