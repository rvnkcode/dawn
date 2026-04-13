use dawn::domain::task::{Description, Index, Task};
use tabled::Tabled;

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct NextRow {
    #[tabled(rename = "ID")]
    pub id: Index,
    pub description: Description,
    // TODO: Age
}
