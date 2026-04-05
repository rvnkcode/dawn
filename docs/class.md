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
      +new(raw) Result~Self, IndexError~
    }

    class Description {
      -String
      +new(&raw) Result~Self, DescriptionEmptyError~
    }

    class Timestamp {
      -i64
      +new(raw) Result~Self, TimestampError~
    }

    class Status {
      <<enumeration>>
      +Pending
      +Completed
      +Deleted
    }

    class Task {
      +UniqueID uid
      +Option~Index~ index
      +Description description
      +Timestamp entry
      +Option~Timestamp~ completed
      +Option~Timestamp~ deleted
    }

    class Filter {
      -HashSet~UniqueID~ uids
      -HashSet~Index~ indices
      -HashSet~Status~ statuses
      +new() Self
      +with_uids(self, uids) Self
      +with_indices(self, indices) Self
      +with_statuses(self, statuses) Self
      +uids(&self) &HashSet~UniqueID~
      +indices(&self) &HashSet~Index~
      +statuses(&self) &HashSet~Status~
      +is_empty(&self) bool
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
      +next(&self)* Result~Vec~Task~~
    }

    class TaskRepository {
      <<interface>>
      +create_task(&self, &id, &req)* Result~_~
      +list_tasks(&self, &filter)* Result~Vec~Task~~
    }
  }

  Task *-- UniqueID
  Task o-- Index
  Task *-- Description
  Task *-- Timestamp
  Task o-- Timestamp
  Filter o-- UniqueID
  Filter o-- Index
  Filter o-- Status
  TaskCreation *-- Description
  Service~R~ ..> UniqueID : generates
  Service~R~ ..> Filter : determines
  Service~R~ ..|> TaskService : implements
  Service~R~ ..> TaskRepository : where R is TaskRepository
  TaskService ..> TaskCreation : accepts
  TaskRepository ..> UniqueID : accepts
  TaskRepository ..> TaskCreation : accepts
  TaskRepository ..> Filter : accepts

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
