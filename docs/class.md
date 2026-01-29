---
title: Class Diagram
---

```mermaid
classDiagram
  namespace Inbound {
    class Cli {
      +new() Self
      +default() Self
      +handle_command(&self) Result~_~
    }
  }
```
