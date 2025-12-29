---
title: Class Diagram
---

```mermaid
classDiagram
  namespace Inbound {
    class Commands {
      <<enumeration>>
      -Add(Modification)
      -All(Modification)
      -Modify(Modification)
    }
    class Modification {
      +Vec~String~ description
    }
    class Handler~TS~ {
      -TS task_service
      +new(task_service) Self
      +add(&self, &raw_filters, &args) Result~_~
      +next(&self, &raw_filters) Result~_~
      +all(&self, &raw_filters, &args) Result~_~
      +modify(&self, &raw_filters, &args) Result~_~
      -display_table~R~(&self, tasks) Result~_~
      -$has_changes(&Task, &TaskModification) bool
      -$get_display_id(&Task) String
      -$print_modify_result(count)
      -$print_diff(&Task, &TaskModification)
      -$confirm_bulk(&display_id, &description) Result~ConfirmResult~
    }
    class Cli {
      -Vec~String~ filters
      -Option~Commands~ command
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
    class ConfirmResult {
      <<enumeration>>
      Yes
      No
      All
      Quit
    }
  }
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
      +UniqueID uid
      +Option~Index~ index
      +Description description
      +i64 created_at
      +Option~i64~ completed_at
      +Option~i64~ deleted_at
    }
    class TaskCreation {
      +Description description
    }
    class TaskModification {
      +Option~Description~ description
      +is_empty(&self) bool
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
      +modify(&self, modification, targets) Result~_~
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
      +update_tasks(&self, modification, targets) Result~_~
    }
  }
  namespace Outbound {
    class SQLite {
      -Connection conn
      +new() Result~Self~
      -$get_path() Result~PathBuf~
      -$open_connection() Result~Connection~
      -$get_user_version(&conn) u8
      -$initialize_schema(&conn) Result~_~
    }
  }

  %% Inbound Flow
  Cli ..> Handler~TS~ : creates
  Cli o-- Commands : has
  Commands *-- Modification : has
  Handler~TS~ o-- TaskService : uses

  %% Handler Use Cases
  Handler~TS~ ..> TaskCreation : creates
  Handler~TS~ ..> TaskModification : creates
  Handler~TS~ ..> Filter : parses
  Handler~TS~ ..> BaseTable~R~ : renders

  %% Table Polymorphism
  BaseTable~R~ o-- TableRow : contains
  TableRow <|.. NextRow : implements
  TableRow <|.. AllRow : implements
  NextRow ..> Task : from
  AllRow ..> Task : from

  %% Domain Entity
  Task *-- UniqueID : has
  Task *-- Description : has

  %% Port Contract
  TaskService ..> Task : returns
  TaskService ..> TaskCreation : accepts
  TaskService ..> TaskModification : accepts
  TaskService ..> Filter : accepts

  %% Hexagonal Implementation
  TaskService <|.. Service~R~ : implements
  Service~R~ --> TaskRepository : delegates
  TaskRepository <|.. SQLite : implements
```
