---
name: coding-standards
description: Universal coding standards, best practices, and patterns for Rust development.
origin: ECC
---

# Coding Standards & Best Practices

Universal coding standards applicable across all projects.

## When to Activate

- Starting a new project or module
- Reviewing code for quality and maintainability
- Refactoring existing code to follow conventions
- Enforcing naming, formatting, or structural consistency
- Setting up linting, formatting, or type-checking rules
- Onboarding new contributors to coding conventions

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
// ✅ GOOD: Immutable by default
let user_name = String::from("Alice");
let items = vec![1, 2, 3];

// ✅ OK: Local mutation for building up state (not exposed via &mut)
let mut headers = HashMap::new();
headers.insert("Content-Type", "application/json");
let headers = headers; // re-bind as immutable once built

// ✅ BETTER: Use iterator chains to avoid mutation entirely
let items: Vec<_> = raw_items.iter().filter(|x| x.is_valid()).collect();

// ❌ BAD: Exposing &mut across module boundaries
pub fn add_item(&mut self, item: Item) { ... } // leaks internal mutability
```

### Error Handling

```rust
// ✅ GOOD: Use thiserror with #[from] for automatic conversion
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("failed to fetch data")]
    Fetch(#[from] FetchError),
    #[error("failed to process data")]
    Process(#[from] ProcessError),
}

fn something_risky() -> Result<(), AppError> {
    let data = fetch_data()?;   // FetchError auto-converts via #[from]
    process_data(data)?;         // ProcessError auto-converts via #[from]
    Ok(())
}

// ❌ BAD: Ignoring errors
fn something_risky() {
    let data = fetch_data().unwrap(); // PANIC on error
    process_data(data).unwrap();      // PANIC on error
}
```

### Async/Await Best Practices

```rust
// ✅ GOOD: Use async for I/O-bound tasks
async fn fetch_user_profile(user_id: &str) -> Result<UserProfile, CustomError> {
    let response = reqwest::get(format!("https://api.example.com/users/{}", user_id)).await?;
    let profile = response.json::<UserProfile>().await?;
    Ok(profile)
}

// ❌ BAD: Blocking calls in async context
async fn fetch_user_profile(user_id: &str) -> Result<UserProfile, CustomError> {
    let response = reqwest::blocking::get(format!("https://api.example.com/users/{}", user_id))?; // BLOCKING call
    let profile = response.json::<UserProfile>()?;
    Ok(profile)
}
```

#### Concurrent Execution

```rust
// ✅ GOOD: Run independent futures concurrently
async fn fetch_dashboard(user_id: &str) -> Result<Dashboard, AppError> {
    let (profile, orders, notifications) = tokio::try_join!(
        fetch_profile(user_id),
        fetch_orders(user_id),
        fetch_notifications(user_id),
    )?;
    Ok(Dashboard { profile, orders, notifications })
}

// ✅ GOOD: Dynamic number of futures
async fn fetch_all_markets(ids: &[&str]) -> Result<Vec<Market>, AppError> {
    let futures: Vec<_> = ids.iter().map(|id| fetch_market(id)).collect();
    let markets = futures::future::try_join_all(futures).await?;
    Ok(markets)
}

// ❌ BAD: Unnecessary sequential execution
async fn fetch_dashboard(user_id: &str) -> Result<Dashboard, AppError> {
    let profile = fetch_profile(user_id).await?;       // waits...
    let orders = fetch_orders(user_id).await?;          // then waits...
    let notifications = fetch_notifications(user_id).await?; // then waits...
    Ok(Dashboard { profile, orders, notifications })
}
```

### Type Safety

```rust
// ✅ GOOD: Newtype pattern to prevent primitive obsession
struct UserId(String);
struct MarketId(String);

fn get_user(id: &UserId) -> Result<User, AppError> { ... }

// ❌ BAD: Bare primitives — easy to mix up arguments
fn get_user(id: &str) -> Result<User, AppError> { ... }
```

```rust
// ✅ GOOD: Enum for known variants
enum MarketStatus {
    Open,
    Closed,
    Suspended,
}

// ❌ BAD: Stringly-typed — typos become runtime bugs
fn set_status(market: &mut Market, status: &str) { ... }
```

```rust
// ✅ GOOD: Bounded generics preserve type information
fn process<T: Serialize + DeserializeOwned>(item: &T) -> Result<(), AppError> { ... }

// ❌ BAD: Type erasure loses compile-time safety
fn process(item: &dyn Any) -> Result<(), AppError> { ... }
```

## Comments & Documentation

### When to Comment

```rust
// ✅ GOOD: Explain WHY, not WHAT
// Using a HashMap for O(1) lookups instead of a Vec for performance
let user_map: HashMap<String, User> = users.into_iter()
    .map(|u| (u.id.clone(), u))
    .collect();

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
/// # Returns
///
/// Markets sorted by descending similarity score.
///
/// # Errors
///
/// Returns [`AppError::Embedding`] if the embedding API fails,
/// or [`AppError::Cache`] if Redis is unavailable.
///
/// # Examples
///
/// (add doc-test code block here with realistic usage)
pub async fn search_markets(
    query: &str,
    limit: usize,
) -> Result<Vec<Market>, AppError> {
    // Implementation
}
```

#### Rustdoc References

- [What is rustdoc? - The rustdoc book](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)
- [References - The rustdoc book](https://doc.rust-lang.org/rustdoc/references.html)

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
                if has_permission(&user, &market) {
                    // Do something
                }
            }
        }
    }
}

// ✅ GOOD: Early returns with let-else (Rust 1.65+)
let Some(user) = get_user() else { return };
if !user.is_admin { return; }
let Some(market) = get_market() else { return };
if !market.is_active { return; }
if !has_permission(&user, &market) { return; }

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

**Remember**: Code quality is not negotiable. Clear, maintainable code enables rapid development and confident refactoring.
