mod age;
pub use age::Age;
mod base_table;
pub use base_table::{BaseTable, TableRow};
mod all_row;
pub use all_row::AllRow;
mod next_row;
pub use next_row::NextRow;
mod status;
pub use status::Status;

pub type NextTable = BaseTable<NextRow>;
pub type AllTable = BaseTable<AllRow>;
