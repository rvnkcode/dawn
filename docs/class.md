---
title: Class Diagram
---

```mermaid
classDiagram
  AppContext~TS~ o-- TaskService
  Cli ..> AppContext~TS~
  Cli ..> Handler~TS~
  Cli o-- Commands
  Handler~TS~ o-- AppContext~TS~
  Handler~TS~ ..> Task
  Handler~TS~ ..> Description
  Commands ..> Modification
  namespace Inbound {
    class AppContext~TS~ {
      +TS task_service
      +new(task_service) Self
    }
    class Commands {
      <<enumeration>>
      -Add(Modification)
      -All(Modification)
    }
    class Modification {
      +Vec~String~ description
    }
    class Handler~TS~ {
      -AppContext~TS~ context
      +new(context) Self
      +add(&self, &filters, &args) Result~_~
      -compose_description(&filters, &description) Result~Description~
      -display_table~R~(&self, tasks) Result~_~
      +next(&self, &raw_filters) Result~_~
      +all(&self, &raw_filters, &args) Result~_~
    }
    class Cli {
      -Vec~String~ filters
      -Options~Commands~ command
      +new() Self
      +handle_command(&self, task_service) Result~_~
    }
    class Age {
      +new(&created_at, &now) Result~Self, AgeError~
    }
    class TableRow {
      <<interface>>
      +new(task, &now) Result~Self~
    }
    class NextRow {
      +Index id
      +Age age
      +Description description
    }
    class Status {
      <<enumeration>>
      Pending
      Completed
      Deleted
    }
    class AllRow {
      +Option~Index~ id
      +Status status
      +UniqueID uid
      +Age age
      +Option~Age~ done
      +Description description
    }
    class BaseTable~R~ {
      -Vec~R~ rows
      +new(tasks) Result~Self~
      +len(&self) usize
      +render(&self) Table
    }
  }
  Task *.. UniqueID
  Task *.. Index
  Task *.. Description
  TaskCreation *.. Description
  Handler~TS~ ..> TaskCreation
  TaskService ..> TaskCreation
  TaskRepository ..> TaskCreation
  TableRow <|.. NextRow
  TableRow <|.. AllRow
  NextRow *.. Index
  NextRow *.. Age
  NextRow *.. Description
  NextRow ..> Task
  AllRow *.. Status
  AllRow *.. UniqueID
  AllRow *.. Age
  AllRow *.. Description
  AllRow ..> Task
  BaseTable~R~ o-- TableRow
  Handler~TS~ ..> BaseTable~R~
  Handler~TS~ ..> Filter
  TaskService <|.. Service
  Service~R~ --> TaskRepository
  TaskRepository <|.. SQLite
  TaskService ..> Task
  TaskService ..> Filter
  TaskRepository ..> Task
  TaskRepository ..> Filter
  Filter o-- Index
  Filter o-- IndexRange
  Filter o-- UniqueID
  IndexRange *-- Index
  namespace Domain {
    class Description {
      +new(raw) Result~Self, DescriptionEmptyError~
    }
    class UniqueID {
      +new() Self
      +from_str(raw) Result~Self, UniqueIDLengthError~
    }
    class Index {
      +new(raw) Result~Self, IndexError~
      +get(&self) usize
    }
    class Task {
      +UniqueId uid
      +Option~Index~ index
      +Description description
      +i64 created_at
      +Option~i64~ completed_at
      +Option~i64~ deleted_at
    }
    class TaskCreation {
      +Description description
    }
    class Filter {
      +Vec~Index~ indices
      +Vec~IndexRange~ ranges
      +Vec~UniqueID~ uids
      +Vec~String~ words
      +is_empty(&self) bool
    }
    class IndexRange {
      -Index start
      -Index end
      +new(a, b) Result~Self, IndexRangeError~
      +start(&self) &Index
      +end(&self) &Index
    }
    class TaskService {
      <<interface>>
      +add(&self, req) Result~_~
      +count_pending(&self) usize
      +next(&self, &filter) Result~Vec~Task~~
      +all(&self, &filter) Result~Vec~Task~~
    }
    class Service~R~ {
      -R repo
      +new(repo) Self
    }
    class TaskRepository {
      <<interface>>
      +create_task(&self, id, req) Result~_~
      +count_pending_tasks(&self) usize
      +get_pending_tasks(&self, &filter) Result~Vec~Task~~
      +get_all_tasks(&self, &filter) Result~Vec~Task~~
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
