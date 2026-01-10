---
title: Class Diagram
---

```mermaid
classDiagram
  namespace Domain {
    class Description {
      +new(&raw) Result~Self, DescriptionEmptyError~
    }
  }
  namespace Outbound {
    class SQLite {
      -Connection conn
      +new() Result~Self~
      -get_path()$ Result~PathBuf~
      -connect()$ Result~Connection~
      -get_user_version(&conn)$ u8
      -initialize_schema(&conn)$ Result~_~
    }
  }
```
