---
title: Class Diagram
---

```mermaid
  namespace Outbound {
    class SQLite {
      -Connection conn
      +new() Result~Self, SQLiteError~
      -get_user_version(&self) Result~u8, SQLiteError~
      +initialize(&mut self) Result~_, SQLiteError~
    }
  }
```
