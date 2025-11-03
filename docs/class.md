---
title: Class Diagram
---

```mermaid
  AppContext~TS~ --> TaskService
  Cli ..> AppContext~TS~
  Cli ..> TaskService
  namespace Inbound {
    class AppContext~TS~ {
      -TS task_service
    }
    class Cli {
      +new() Self
      +handle_command(&self, task_service) Result~_~
    }
  }
  TaskService <|.. Service
  Service~R~ --> TaskRepository
  TaskRepository <|.. SQLite
  namespace Domain {
    class TaskService {
      <<interface>>
    }
    class Service~R~ {
      -R repo
      +new(repo) Self
    }
    class TaskRepository {
      <<interface>>
    }
  }
  namespace Outbound {
    class SQLite {
      -Connection conn
      +new() Result~Self~
      -get_path() Result~PathBuf~
      -open_connection() Result~Connection~
      -get_user_version(&conn) u8
      -initialize_schema(&conn) Result~_~
    }
  }
```
