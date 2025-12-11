/// Test utilities shared across test modules
#[cfg(test)]
pub fn strs(arr: &[&str]) -> Vec<String> {
    arr.iter().map(|s| s.to_string()).collect()
}
