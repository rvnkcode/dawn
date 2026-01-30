---
name: coding-standards
description: Universal coding standards, best practices, and patterns for Rust development.
---

# Coding Standards & Best Practices

Universal coding standards applicable across all projects.

## Code Quality Principles

### 1. Readability First

- Code is read more than written
- Clear variable and function names
- Self-documenting code preferred over comments
- Consistent formatting

### 2. KISS (Keep It Simple, Stupid)

- Simplest solution that works
- Avoid over-engineering
- No premature optimization
- Easy to understand > clever code

### 3. DRY (Don't Repeat Yourself)

- Extract common logic into functions
- Create reusable components
- Share utilities across modules
- Avoid copy-paste programming

### 4. YAGNI (You Aren't Gonna Need It)

- Don't build features before they're needed
- Avoid speculative generality
- Add complexity only when required
- Start simple, refactor when needed

## Rust Standards

### Variable Naming

```rust
// ✅ GOOD: Descriptive names
let market_search_query = "election";
let is_user_authenticated = true;
let total_revenue = 1000;

// ❌ BAD: Unclear names
let q = "election";
let flag = true;
let x = 1000;
```

### Function Naming

```rust
// ✅ GOOD: Verb-noun pattern
fn fetch_market_data(market_id: &str) { }
fn calculate_similarity(a: &[f32], b: &[f32]) -> f32 { }
fn is_valid_email(email: &str) -> bool { }

// ❌ BAD: Unclear or noun-only
fn market(id: &str) { }
fn similarity(a: &[f32], b: &[f32]) -> f32 { }
fn email(e: &str) { }
```

### Immutability Pattern (CRITICAL)

```rust
// ✅ ALWAYS prefer immutability
let user_name = String::from("Alice");
let items = vec![1, 2, 3];

// ❌ NEVER use mutable unless necessary
let mut user_name = String::from("Alice");  // BAD
let mut items = vec![1, 2, 3];               // BAD
```

### Error Handling

```rust
// ✅ GOOD: Comprehensive error handling
fn fetch_data(url: &str) -> Result<String, CustomError> {
    let response = request::blocking::get(url)?;
    match response.status() {
        StatusCode::OK => Ok(response.text()?),
        status => Err(CustomError::HttpError(status)),
    }
}
```

## Comments & Documentation

### When to Comment

```rust
// ✅ GOOD: Explain WHY, not WHAT
// Using a HashMap for O(1) lookups instead of a Vec for performance
let mut user_map: HashMap<String, User> = HashMap::new();

// Deliberately using mutation here for performance with large datasets
user_map.insert(user.id.clone(), user);

// ❌ BAD: Stating the obvious
// Increment counter by 1
counter += 1;

// Set name to user's name
name = user.name.clone();
```

### Documentation for Public APIs

```rust
/// Searches markets using semantic similarity.
///
/// # Arguments
/// * `query` - Natural language search query
/// * `limit` - Maximum number of results (default: 10)
/// # Returns
/// Array of markets sorted by similarity score
/// # Errors
/// Returns `CustomError` if OpenAI API fails or Redis unavailable
pub fn search_markets(
    query: &str,
    limit: usize,
) -> Result<Vec<Market>, CustomError> {
    // Implementation
}
```

## Performance Best Practices

### Database Queries

```rust
// ✅ GOOD: Select only needed columns
let stmt = conn.prepare("SELECT id, name, status FROM markets LIMIT 10")?;
let market_iter = stmt.query_map([], |row| {
    Ok(Market {
        id: row.get(0)?,
        name: row.get(1)?,
        status: row.get(2)?,
    })
})?;

// ❌ BAD: Select everything
let stmt = conn.prepare("SELECT * FROM markets")?;
let market_iter = stmt.query_map([], |row| {
    Ok(Market {
        id: row.get(0)?,
        name: row.get(1)?,
        status: row.get(2)?,
        description: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
})?;
```

## Testing Standards

### Test Structure (AAA Pattern)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_similarity() {
        // Arrange
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![0.0, 1.0, 0.0];

        // Act
        let similarity = calculate_cosine_similarity(&vector1, &vector2);

        // Assert
        assert_eq!(similarity, 0.0);
    }
}

```

### Test Naming

```rust
// ✅ GOOD: Descriptive test names
#[test]
fn returns_empty_array_when_no_markets_match_query() { }
#[test]
fn throws_error_when_openai_api_key_is_missing() { }
#[test]
fn falls_back_to_substring_search_when_redis_unavailable() { }

// ❌ BAD: Vague test names
#[test]
fn works() { }
#[test]
fn test_search() { }
```

## Code Smell Detection

Watch for these anti-patterns:

### 1. Long Functions

```rust
// ❌ BAD: Function > 50 lines
fn process_market_data() {
    // 100 lines of code
}

// ✅ GOOD: Split into smaller functions
fn process_market_data() {
    let validated = validate_data();
    let transformed = transform_data(validated);
    save_data(transformed);
}
```

### 2. Deep Nesting

```rust
// ❌ BAD: 5+ levels of nesting
if let Some(user) = get_user() {
    if user.is_admin {
        if let Some(market) = get_market() {
            if market.is_active {
                if has_permission(user, market) {
                    // Do something
                }
            }
        }
    }
}

// ✅ GOOD: Early returns
let user = match get_user() {
    Some(u) => u,
    None => return,
};
if !user.is_admin { return; }
let market = match get_market() {
    Some(m) => m,
    None => return,
};
if !market.is_active { return; }
if !has_permission(user, market) { return; }

// Do something
```

### 3. Magic Numbers

```rust
// ❌ BAD: Unexplained numbers
if retry_count > 3 { }
std::thread::sleep(std::time::Duration::from_millis(500));

// ✅ GOOD: Named constants
const MAX_RETRIES: u32 = 3;
const DEBOUNCE_DELAY_MS: u64 = 500;

if retry_count > MAX_RETRIES { }
std::thread::sleep(std::time::Duration::from_millis(DEBOUNCE_DELAY_MS));
```
