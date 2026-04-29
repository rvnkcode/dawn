---
title: Class Diagram
---

```mermaid
direction BT
  namespace CLI {
    class Creation {
      ~Vec~String~ description
    }

    class Modification {
      ~Vec~String~ mods
    }

    class Parsed {
      -HashSet~UniqueID~ uids
      -HashSet~Index~ indices
      -Vec~String~ words
      -bool has_bare_id
    }

    class DefaultCommand {
      <<enumeration>>
      ~Next(Filter)
      ~Info(Filter)
    }

    class Action {
      <<enumeration>>
      ~Modify
    }

    class Command {
      <<enumeration>>
      -Add(Creation)
      -Modify(Modification)
    }

    class Cli {
      -Vec~String~ filter
      -Option~Command~ command
      +new() Self
      +handle_command(&self, task_service) Result~_, CliError~
    }

    class Handler~TS~ {
      -TS task_service
      ~new(task_service) Self
      ~add(&self, &filter, &words) Result~_, CliError~
      ~default(&self, &raw_filters) Result~_, CliError~
      -next(&self, filter) Result~_, CliError~
      -info(&self, &filter) Result~_, CliError~
      ~modify(&self, &raw_filter, &mods) Result~_, CliError~
      ~done(&self, &raw_filter, &mods) Result~_, CliError~
    }

    class Age {
      -String
      ~new(&entry, now) Result~Self, AgeError~
    }

    class TableRow {
      <<interface>>
      ~new(task, now)* Result~Self~
    }

    class NextRow {
      -Index id
      -Age age
      -Description description
      ~new(task, now) Result~Self~
    }

    class BaseTable~R~ {
      -Vec~R~ rows
      ~new(tasks) Result~Self~
      ~count(&self) usize
      ~render(&self) Table
    }

    class InfoRow {
      -String name
      -String value
    }

    class InfoTable {
      -Vec~InfoRow~ rows
      ~new(&task, now) Result~Self~
      ~render(&self) Table
    }
  }

  Cli ..> TaskService : accepts
  Cli ..> Handler : calls
  Handler~TS~ ..> TaskService : where TS is TaskService

  %% Create
  Command *-- Creation : has
  Cli ..> Creation : parses
  Cli o-- Command : executes
  Handler~TS~ ..> Description : creates
  Handler~TS~ ..> TaskCreation : creates

  %% Read
  Age ..> Timestamp : accepts
  NextRow *-- Index
  NextRow *-- Age
  NextRow *-- Description
  NextRow ..|> TableRow : implements
  BaseTable~R~ ..> TableRow : where R is TableRow and Tabled
  BaseTable~R~ ..> Task : accepts
  InfoTable o-- InfoRow
  InfoTable ..> Task : accepts
  DefaultCommand *-- Filter : carries
  Parsed o-- UniqueID
  Parsed o-- Index
  Handler~TS~ ..> DefaultCommand : dispatches
  Handler~TS~ ..> Filter : parses and accepts
  Handler~TS~ ..> Status : targets
  Handler~TS~ ..> Task : fetches
  Handler~TS~ ..> BaseTable~R~ : displays
  Handler~TS~ ..> NextRow : displays
  Handler~TS~ ..> InfoTable : displays

  %% Update
  Command *-- Modification : has
  Cli ..> Modification : parses
  Handler~TS~ ..> TaskModification : creates
  Handler~TS~ ..> Action : drives

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
      +as_seconds(&self) i64
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
      +Timestamp modified
      +status(&self) Status
    }

    class Filter {
      -HashSet~UniqueID~ uids
      -HashSet~Index~ indices
      -HashSet~Status~ statuses
      -Vec~String~ words
      +with_uids(mut self, uids) Self
      +with_indices(mut self, indices) Self
      +with_statuses(mut self, statuses) Self
      +with_words(mut self, words) Self
      +uids(&self) &HashSet~UniqueID~
      +indices(&self) &HashSet~Index~
      +statuses(&self) &HashSet~Status~
      +words(&self) &[String]
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
      +count_pending(&self)* Result~usize~
      +list(&self, &filter)* Result~Vec~Task~~
      +modify(&self, &modification, &targets)* Result~_~
      +purge(&self, &targets)* Result~_~
    }

    class TaskRepository {
      <<interface>>
      +create_task(&self, &id, &req)* Result~_~
      +count_pending(&self)* Result~usize~
      +list_tasks(&self, &filter)* Result~Vec~Task~~
      +update_tasks(&self, &modification, &targets)* Result~_~
      +delete_tasks(&self, &targets)* Result~_~
    }
  }

  %% Adapters
  Service~R~ ..|> TaskService : implements
  Service~R~ ..> TaskRepository : where R is TaskRepository

  %% Create
  TaskCreation *-- Description
  TaskService ..> TaskCreation : accepts
  Service~R~ ..> UniqueID : generates
  TaskRepository ..> UniqueID : accepts
  TaskRepository ..> TaskCreation : accepts

  %% Read
  Task *-- UniqueID
  Task o-- Index
  Task *-- Description
  Task *-- Timestamp : entry, modified
  Task o-- Timestamp : completed, deleted
  Task ..> Status : computes
  Filter o-- UniqueID
  Filter o-- Index
  Filter o-- Status
  TaskService ..> Task : returns
  TaskRepository ..> Filter : accepts
  TaskRepository ..> Task : returns

  %% Update
  TaskModification o-- Description
  TaskModification o-- Timestamp
  TaskService ..> UniqueID : accepts
  TaskService ..> TaskModification : accepts
  TaskRepository ..> TaskModification : accepts

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
