# Security Guidelines

## Mandatory Security Checks

Before ANY commit:

- [ ] No hardcoded secrets (API keys, passwords, tokens)
- [ ] All user inputs validated
- [ ] SQL injection prevention (parameterized queries)
- [ ] Error messages don't leak sensitive data

## Secret Management

```rust
// NEVER: Hardcoded secrets
let api_key = "sk-proj-xxxxx";

// ALWAYS: Environment variables
let api_key = std::env::var("API_KEY").expect("API_KEY not configured");
```
