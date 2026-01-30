---
name: tdd-workflow
description:  Use this skill when writing new features, fixing bugs, or refactoring code. Enforces test-driven development with 80%+ coverage including unit, integration, and E2E tests.
---

# Test-Driven Development (TDD) Workflow

This skill ensures all code development follows TDD principles with comprehensive test coverage.

## When to Activate

- Writing new features or functionality
- Fixing bugs or issues
- Refactoring existing code
- Adding CLI commands
- Creating new components

## Core Principles

### 1. Tests BEFORE Code

ALWAYS write tests first, then implement code to make tests pass.

### 2. Coverage Requirements

- Minimum 80% coverage (unit + integration + E2E)
- All edge cases covered
- Error scenarios tested
- Boundary conditions verified

### 3. Test Types

#### Unit Tests

- Individual functions and utilities
- Component logic
- Pure functions
- Helpers and utilities

#### Integration Tests

- CLI commands
- Database operations
- Service interactions
- External API calls

#### E2E Tests (Playwright)

- Critical user flows
- Complete workflows
- Browser automation
- UI interactions

## TDD Workflow Steps

### Step 1: Write User Journeys

```
As a [role], I want to [action], so that [benefit]

Example:
As a user, I want to search for markets semantically,
so that I can find relevant markets even without exact keywords.
```

### Step 2: Generate Test Cases

For each user journey, create comprehensive test cases:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_relevant_markets_for_query() {
        // Test implementation
    }
    #[test]
    fn handles_empty_query_gracefully() {
        // Test edge case
    }
    #[test]
    fn falls_back_to_substring_search_when_redis_unavailable() {
        // Test fallback behavior
    }
    #[test]
    fn sorts_results_by_similarity_score() {
        // Test sorting logic
    }
}
```

### Step 3: Run Test (They Should Fail)

```bash
cargo t
# Test should fail - we haven't implemented yet
```

### Step 4: Implement Code

Write minimal code to make tests pass:

```rust
pub fn search_markets(query: &str) -> Vec<Market> {
    // Minimal implementation to pass tests
}
```

### Step 5: Run Tests Again

```bash
cargo t
# Test should now pass
```

### Step 6: Refactor

Improve code quality while keeping tests green:

- Remove duplication
- Improve names
- Optimize performance
- Enhance readability

### Step 7: Verify Coverage

```bash
cargo llvm-cov  # Verify 80%+ coverage
```

## Mocking Adapters (Hexagonal Architecture)

Ports (traits) define contracts; mock adapters implement them for testing.

```rust
// 1. Port (trait)
pub trait TaskRepository: Send + Sync {
    fn save(&self, task: &Task) -> Result<(), Error>;
    fn find_by_id(&self, id: &str) -> Result<Option<Task>, Error>;
}

// 2. Mock adapter
#[derive(Default)]
struct MockTaskRepository {
    tasks: RefCell<HashMap<String, Task>>,
    should_fail: Cell<bool>,
}

impl TaskRepository for MockTaskRepository {
    fn save(&self, task: &Task) -> Result<(), Error> {
        if self.should_fail.get() { return Err(Error::Mock); }
        self.tasks.borrow_mut().insert(task.id.clone(), task.clone());
        Ok(())
    }
    // ... other methods
}

// 3. Test with mock
#[test]
fn test_service_with_mock() {
    let mock = Arc::new(MockTaskRepository::default());
    let service = TaskService::new(mock.clone());

    service.create_task("Test").unwrap();
    assert!(mock.tasks.borrow().contains_key("1"));
}

#[test]
fn test_error_handling() {
    let mock = Arc::new(MockTaskRepository::default());
    mock.should_fail.set(true);
    let service = TaskService::new(mock);

    assert!(service.create_task("Test").is_err());
}
```

### Using `mockall` Crate

```rust
use mockall::automock;

#[automock]
pub trait TaskRepository { /* ... */ }

#[test]
fn test_with_mockall() {
    let mut mock = MockTaskRepository::new();
    mock.expect_save().returning(|_| Ok(()));

    let service = TaskService::new(Arc::new(mock));
    // ...
}
```

## Edge Cases MUST Test

1. **Null/Undefined**: What if input is null?
2. **Empty**: What if array/string is empty?
3. **Invalid Types**: What if wrong type passed?
4. **Boundaries**: Min/max values
5. **Errors**: Network failures, database errors
6. **Race Conditions**: Concurrent operations
7. **Large Data**: Performance with 10k+ items
8. **Special Characters**: Unicode, emojis, SQL characters

## Test Quality Checklist

Before marking tests complete:

- [ ] All public functions have unit tests
- [ ] All CLI commands have integration tests
- [ ] Critical user flows have E2E tests
- [ ] Edge cases covered (null, empty, invalid)
- [ ] Error paths tested (not just happy path)
- [ ] Mocks used for external dependencies
- [ ] Tests are independent (no shared state)
- [ ] Test names describe what's being tested
- [ ] Assertions are specific and meaningful
- [ ] Coverage is 80%+ (verify with coverage report)

## Common Testing Mistakes to Avoid

### ❌ WRONG: Testing Implementation Details

```rust
// DON'T test internal state
assert_eq!(component.state.count, 5);
```

### ✅ CORRECT: Test User-Visible Behavior

```rust
// DO test what users see
assert_eq!(screen.get_text(), "Count: 5");
```

### ❌ WRONG: Brittle Selectors

```typescript
// Breaks easily
await page.click('.css-class-xyz')
```

### ✅ CORRECT: Semantic Selectors

```typescript
// Resilient to changes
await page.click('button:has-text("Submit")')
await page.click('[data-testid="submit-button"]')
```

### ❌ WRONG: No Test Isolation

```rust
// Test depends on each other
#[test]
fn test_creates_user() { /* ... */ }
#[test]
fn test_updates_same_user() { /* needs previous test */ }
```

### ✅ CORRECT: Independent Tests

```rust
// Each test sets up its own data
#[test]
fn test_creates_user() {
    let user = createTestUser();
    // Test logic
}
```

## Coverage Report

```bash
# Install (once)
cargo install cargo-llvm-cov

# Run coverage
cargo llvm-cov           # Terminal summary
cargo llvm-cov --html    # HTML report (target/llvm-cov/html/)
cargo llvm-cov --lcov    # For CI (codecov, coveralls)
```

## Best Practices

1. **Write Tests First** - Always TDD
2. **One Assert Per Test** - Focus on single behavior
3. **Descriptive Test Names** - Explain what's tested
4. **Arrange-Act-Assert** - Clear test structure
5. **Mock External Dependencies** - Isolate unit tests
6. **Test Edge Cases** - Null, undefined, empty, large
7. **Test Error Paths** - Not just happy paths
8. **Keep Tests Fast** - Unit tests < 50ms each
9. **Clean Up After Tests** - No side effects
10. **Review Coverage Reports** - Identify gaps

## Success Metrics

- 80%+ code coverage achieved
- All tests passing (green)
- No skipped or disabled tests
- Fast test execution (< 30s for unit tests)
- E2E tests cover critical user flows
- Tests catch bugs before production
