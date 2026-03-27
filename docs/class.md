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

    class Index {
      -usize
      +new() Result~Self, IndexError~
    }

    class Description {
      -String
      +new(&raw) Result~Self, DescriptionEmptyError~
    }

    class Timestamp {
      -i64
      +new() Result~Self, TimestampError~
    }

    class Status {
      <<enumeration>>
      +Pending
      +Completed
      +Deleted
    }

    class Task {
      +UniqueID id
      +Option~Index~ index
      +Description description
      +Timestamp entry
      +Option~Timestamp~ completed
      +Option~Timestamp~ deleted
    }

    class Filter {
      +HashSet~Status~ statuses
      +is_empty() bool
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

  Task *-- UniqueID
  Task o-- Index
  Task *-- Description
  Task *-- Timestamp
  Task o-- Timestamp
  Filter o-- Status
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
