use crate::domain::task::Index;

// TODO: Other filter properties
#[derive(Default)]
pub struct Filter {
    pub indices: Vec<Index>,
}

impl Filter {
    pub fn new(indices: Vec<Index>) -> Self {
        Self { indices }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}
