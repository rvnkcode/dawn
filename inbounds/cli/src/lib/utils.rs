#[cfg(test)]
use dawn::domain::task::Task;

/// Test utilities shared across test modules
#[cfg(test)]
pub fn strs(arr: &[&str]) -> Vec<String> {
    arr.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
pub fn make_task(desc: &str, index: Option<usize>, completed: bool, deleted: bool) -> Task {
    use dawn::domain::task::{Description, Index, UniqueID};

    Task {
        uid: "abc12345678".parse::<UniqueID>().unwrap(),
        index: index.map(|i| Index::new(i).unwrap()),
        description: Description::new(desc).unwrap(),
        created_at: 0,
        completed_at: if completed { Some(1234567890) } else { None },
        deleted_at: if deleted { Some(1234567890) } else { None },
    }
}
