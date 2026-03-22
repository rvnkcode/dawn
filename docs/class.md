---
title: Class Diagram
---

```mermaid
  namespace Domain {
    class UniqueID {
      -String
      +new() Self
    }

    class Description {
      -String
      +new(&raw) Result~Self, DescriptionEmptyError~
    }

    class TaskCreation {
      +Description description
    }
  }

  TaskCreation *-- Description

  namespace Outbound {
    class SQLite {
      -Connection conn
      +new() Result~Self, SQLiteError~
      -get_user_version(&self) Result~u8, SQLiteError~
      +initialize(&mut self) Result~_, SQLiteError~
    }
  }
```
