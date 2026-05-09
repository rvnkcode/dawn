#[derive(Debug, PartialEq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq)]
pub enum SortKey {
    Entry(Direction),
    Completed(Direction),
}
