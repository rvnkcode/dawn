use dawn::domain::task::{Description, Index, Task, Timestamp};
use uuid::Uuid;

pub(crate) fn task(index: Option<Index>, description: &str, entry_secs: i64) -> Task {
    Task {
        uuid: Uuid::new_v4(),
        index,
        description: Description::new(description).unwrap(),
        entry: Timestamp::new(entry_secs).unwrap(),
        completed: None,
        deleted: None,
        modified: Timestamp::new(entry_secs).unwrap(),
    }
}
