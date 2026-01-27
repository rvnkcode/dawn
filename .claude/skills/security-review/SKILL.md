---
name: security-review
description: Use this skill when adding authentication, handling user input, working with secrets, creating API endpoints, or implementing payment/sensitive features. Provides comprehensive security checklist and patterns.
---

# Security Review Skill

This skill ensures all code follows security best practices and identifies potential vulnerabilities.

## When to Activate

- Implementing authentication or authorization
- Handling user input or file uploads
- Creating new API endpoints/CLI commands
- Working with secrets or credentials
- Storing or transmitting sensitive data
- Integrating third-party APIs

## Security Checklist

### 1. Secrets Management

#### NEVER Do This

```rust
let api_key = "sk-proj-xxxxx";  // Hardcoded secret
let db_password = "password123"; // In source code
```

#### ALWAYS Do This

```rust
let api_key = std::env::var("API_KEY").expect("API_KEY not configured");
let db_password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD not configured");
```

#### Verification Steps

- [ ] No hardcoded API keys, tokens, or passwords
- [ ] All secrets in environment variables
- [ ] `.env.local` in .gitignore
- [ ] No secrets in git history

### 2. Input Validation

#### Always Validate User Input

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("invalid email format")]
    InvalidEmail,
    #[error("name must be 1-100 characters")]
    InvalidName,
    #[error("age must be 0-150")]
    InvalidAge,
}

pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub age: u8,
}

impl CreateUserInput {
    pub fn validate(email: &str, name: &str, age: u8) -> Result<Self, ValidationError> {
        if !email.contains('@') {
            return Err(ValidationError::InvalidEmail);
        }
        if name.is_empty() || name.len() > 100 {
            return Err(ValidationError::InvalidName);
        }
        if age > 150 {
            return Err(ValidationError::InvalidAge);
        }
        Ok(Self {
            email: email.to_string(),
            name: name.to_string(),
            age,
        })
    }
}
```

#### File Upload Validation

```rust
use std::path::Path;

const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5MB
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif"];

pub fn validate_file_upload(path: &Path, size: u64) -> Result<(), &'static str> {
    // Size check
    if size > MAX_FILE_SIZE {
        return Err("File too large (max 5MB)");
    }

    // Extension check (whitelist)
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match extension {
        Some(ext) if ALLOWED_EXTENSIONS.contains(&ext.as_str()) => Ok(()),
        _ => Err("Invalid file extension"),
    }
}
```

#### Verification Steps

- [ ] All user inputs validated with custom types
- [ ] File uploads restricted (size, type, extension)
- [ ] No direct use of user input in queries
- [ ] Whitelist validation (not blacklist)
- [ ] Error messages don't leak sensitive info

### 3. SQL Injection Prevention

#### NEVER Concatenate SQL

```rust
// DANGEROUS - SQL Injection vulnerability
let query = format!("SELECT * FROM users WHERE email = '{}'", user_email);
conn.execute(&query)?;
```

#### ALWAYS Use Parameterized Queries

```rust
// Safe - parameterized query with rusqlite
let user = conn.query_row(
    "SELECT id, email, name FROM users WHERE email = ?1",
    [&user_email],
    |row| {
        Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
        })
    },
)?;

// Or with named parameters
let user = conn.query_row(
    "SELECT id, email, name FROM users WHERE email = :email",
    &[(":email", &user_email)],
    |row| { /* ... */ },
)?;
```

#### Verification Steps

- [ ] All database queries use parameterized queries
- [ ] No string concatenation/formatting in SQL
- [ ] ORM/query builder used correctly

### 4. Sensitive Data Exposure

#### Logging

```rust
// WRONG: Logging sensitive data
tracing::info!("User login: email={}, password={}", email, password);
tracing::info!("Payment: card={}, cvv={}", card_number, cvv);

// CORRECT: Redact sensitive data
tracing::info!("User login: email={}, user_id={}", email, user_id);
tracing::info!("Payment: last4={}, user_id={}", card.last4, user_id);
```

#### Error Messages

```rust
// WRONG: Exposing internal details
fn handle_error(err: AppError) -> Response {
    Response::json(&json!({
        "error": err.to_string(),
        "backtrace": format!("{:?}", err)
    }))
}

// CORRECT: Generic error messages
fn handle_error(err: AppError) -> Response {
    tracing::error!("Internal error: {:?}", err);
    Response::json(&json!({
        "error": "An error occurred. Please try again."
    }))
}
```

#### Verification Steps

- [ ] No passwords, tokens, or secrets in logs
- [ ] Error messages generic for users
- [ ] Detailed errors only in server logs
- [ ] No stack traces exposed to users

### 5. Dependency Security

#### Regular Updates

```bash
# Check for vulnerabilities
cargo audit

# Update dependencies
cargo update
```

#### Lock Files

```bash
# ALWAYS commit lock files
git add Cargo.lock

# CI should use locked dependencies
cargo build --locked
```

#### Verification Steps

- [ ] Dependencies up to date
- [ ] No known vulnerabilities (`cargo audit` clean)
- [ ] `Cargo.lock` committed
- [ ] Dependabot/RenovateBot enabled
- [ ] Regular security updates

## Pre-Deployment Security Checklist

Before ANY production deployment:

- [ ] **Secrets**: No hardcoded secrets, all in env vars
- [ ] **Input Validation**: All user inputs validated
- [ ] **SQL Injection**: All queries parameterized
- [ ] **Authentication**: Proper token handling
- [ ] **Error Handling**: No sensitive data in errors
- [ ] **Logging**: No sensitive data logged
- [ ] **Dependencies**: Up to date, no vulnerabilities
- [ ] **File Uploads**: Validated (size, type)

## Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

