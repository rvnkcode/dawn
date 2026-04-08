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
      ~get(&self) usize
    }

    class Description {
      -String
      +new(&raw) Result~Self, DescriptionEmptyError~
    }

    class Timestamp {
      -i64
      +new(raw) Result~Self, TimestampError~
      ~get(&self) i64
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

    class TaskModification {
      +Option~Description~ description
      +Option~Option~Timestamp~~ completed
      +Option~Option~Timestamp~~ deleted
      +is_empty(&self) bool
    }

    class Service~R~ {
      -R repo
      +new(repo) Self
    }

    class TaskService {
      <<interface>>
      +add(&self, &req)* Result~_~
      +next(&self)* Result~Vec~Task~~
      +modify(&self, &modification, &targets)* Result~_~
      +purge(&self, &targets)* Result~_~
    }

    class TaskRepository {
      <<interface>>
      +create_task(&self, &id, &req)* Result~_~
      +list_tasks(&self, &filter)* Result~Vec~Task~~
      +update_tasks(&self, &modification, &targets)* Result~_~
      +delete_tasks(&self, &targets)* Result~_~
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
  TaskModification o-- Description
  TaskModification o-- Timestamp
  Service~R~ ..> UniqueID : generates
  Service~R~ ..> Filter : determines
  Service~R~ ..|> TaskService : implements
  Service~R~ ..> TaskRepository : where R is TaskRepository
  TaskService ..> UniqueID : accepts
  TaskService ..> TaskCreation : accepts
  TaskService ..> TaskModification : accepts
  TaskService ..> Task : returns
  TaskRepository ..> UniqueID : accepts
  TaskRepository ..> TaskCreation : accepts
  TaskRepository ..> TaskModification : accepts
  TaskRepository ..> Filter : accepts
  TaskRepository ..> Task : returns

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
