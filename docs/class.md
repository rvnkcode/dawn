---
title: Class Diagram
---

```mermaid
direction BT
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

    class Service~R~ {
      -R repo
      +new(repo) Self
    }

    class TaskService {
      <<interface>>
      +add(&self, &req)* Result~_~
    }

    class TaskRepository {
      <<interface>>
      +create_task(&self, &id, &req)* Result~_~
    }
  }

  TaskCreation *-- Description
  Service~R~ ..> UniqueID : generates
  Service~R~ ..|> TaskService : implements
  Service~R~ ..> TaskRepository : where R is TaskRepository
  TaskService ..> TaskCreation : accepts
  TaskRepository ..> UniqueID : accepts
  TaskRepository ..> TaskCreation : accepts

  namespace Outbound {
    class SQLite {
      -Connection conn
      +new() Result~Self, SQLiteError~
      -get_user_version(&self) Result~u8, SQLiteError~
      +initialize(&mut self) Result~_, SQLiteError~
    }
  }

  SQLite ..|> TaskRepository : implements
```
