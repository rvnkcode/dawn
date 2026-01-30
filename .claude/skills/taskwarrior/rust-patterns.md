# C++ to Rust Patterns

Reference for translating Taskwarrior C++ patterns to Rust idioms.

---

## Data Structures

| C++ | Rust |
|-----|------|
| `std::map<K,V>` | `HashMap<K,V>` |
| `std::vector<T>` | `Vec<T>` |
| `std::string` | `String` / `&str` |
| `std::optional<T>` | `Option<T>` |
| Empty string = unset | `None` = unset |

---

## Task Storage

**C++ (map-based):**

```cpp
class Task {
    std::map<std::string, std::string> data;
    std::string get(const std::string& key);
    void set(const std::string& key, const std::string& value);
};
```

**Rust (struct-based):**

```rust
pub struct Task {
    // Known fields as typed members
    pub uuid: Uuid,
    pub status: Status,
    pub description: String,
    pub due: Option<DateTime<Utc>>,

    // UDAs as HashMap
    #[serde(flatten)]
    pub uda: HashMap<String, String>,
}
```

---

## Enums with String Conversion

```rust
use strum::{Display, EnumString};

#[derive(Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Pending,
    Completed,
    Deleted,
}

// Usage
let s: Status = "pending".parse()?;
let name = s.to_string();  // "pending"
```

---

## Inheritance → Traits

**C++:**

```cpp
class Command {
    virtual std::string name() = 0;
    virtual int execute(Context&) = 0;
};
```

**Rust:**

```rust
pub trait Command {
    fn name(&self) -> &'static str;
    fn execute(&self, ctx: &mut Context) -> Result<()>;
}
```

---

## Factory Pattern

```rust
pub struct CommandFactory {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandFactory {
    pub fn get(&self, name: &str) -> Option<Arc<dyn Command>>;
}
```

---

## Error Handling

**C++ (error codes):**

```cpp
int Task::validate() {
    if (data["description"].empty()) {
        context.error("Missing description");
        return 1;
    }
    return 0;
}
```

**Rust (thiserror):**

```rust
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task must have a description")]
    MissingDescription,

    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition { from: Status, to: Status },
}

fn validate(&self) -> Result<(), TaskError> {
    if self.description.is_empty() {
        return Err(TaskError::MissingDescription);
    }
    Ok(())
}
```

---

## Smart Pointers

| C++ | Rust |
|-----|------|
| `std::shared_ptr<T>` | `Arc<T>` |
| `std::unique_ptr<T>` | `Box<T>` |
| Raw pointer | `&T` / `&mut T` |

---

## Trait Implementations

| C++ | Rust |
|-----|------|
| Copy constructor | `Clone` trait |
| `operator==` | `PartialEq` / `Eq` |
| `operator<` | `PartialOrd` / `Ord` |
| `operator<<` | `Display` / `Debug` |

---

## Recommended Crates

| Purpose | Crate |
|---------|-------|
| CLI | `clap` |
| Serialization | `serde`, `serde_json` |
| Date/Time | `chrono` |
| UUID | `uuid` |
| Enum strings | `strum` |
| Error handling | `thiserror`, `anyhow` |
| Tables | `tabled` |
| Parsing | `nom` or `pest` |
