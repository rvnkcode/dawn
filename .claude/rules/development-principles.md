# Development Principles

When implementing features or reviewing code, ensure the following:

## Hexagonal Architecture

Separate business logic from external concerns via ports and adapters:

- **Ports**: Interfaces (traits) that define how the domain interacts with the outside world
- **Adapters**: Concrete implementations of ports (database, HTTP, CLI, etc.)
- **Dependency direction**: Inbound/Outbound → Domain (never Domain → Adapters)
- Adapters implement ports, never the reverse
- **Layers**:
  - `domain/` — Entities, value objects, domain services, use cases, port definitions
  - `inbound/` — Adapters that drive the application (CLI, HTTP handlers, event listeners)
  - `outbound/` — Adapters driven by the application (DB, API clients, file I/O)
- Domain logic MUST NOT depend on frameworks or external libraries directly

## OOP (SOLID)

- **SRP**: Each module/struct has one reason to change
- **OCP**: Extend behavior through new implementations, not by modifying existing code
- **LSP**: Subtypes must be substitutable for their base abstractions without breaking behavior
- **ISP**: Define small, focused traits — clients should not depend on methods they don't use
- **DIP**: Depend on abstractions (traits/interfaces), not concrete types

Additional principles:

- Encapsulation: Access internal state only through behavior (methods), never expose fields directly
- Polymorphism: Generic implementations over trait bounds, program against traits not concrete types
- Composition over inheritance: Combine small, focused types instead of building deep hierarchies

## DDD (Domain-Driven Design)

- **Bounded Context**: Separate modules per business subdomain with explicit boundaries
- **Entity**: Identified by a unique ID, equality based on identity
- **Value Object**: Immutable, equality based on all fields, no identity
- **Aggregate**: Consistency boundary — external code references only the aggregate root
- **Domain Service**: Operations that don't belong to a single entity
- **Repository pattern**: Abstract persistence behind a trait; domain never knows storage details
- **Ubiquitous Language**: Code naming MUST match domain terminology (e.g., `Task`, `TaskId`, `Description`)

## Agile Development

- **Incremental delivery**: Implement in the smallest working unit that provides value
- **Small commits**: Each commit represents one logical change; message describes what and why
- **Small PRs**: Reviewable in under 30 minutes; split large changes into stacked PRs
- **Iterative refinement**: Start simple, refactor as understanding grows
- **YAGNI**: Only implement what is needed right now — no speculative features

## TDD as Design Tool

Tests drive architectural decisions, not just correctness:

- Code that is hard to test signals a design problem — fix the design, not the test
- External dependencies (DB, network, filesystem) are accessed through ports, replaceable with test doubles
- Unit tests target domain logic, integration tests target adapters
- Concrete TDD workflow (RED-GREEN-REFACTOR) is defined in `testing.md`

## CLI POSIX Compliance

- **Exit codes**: `0` = success, `1` = runtime error, `2` = usage/argument error
- **Options**: Support short (`-v`) and long (`--verbose`) forms; `--` terminates option parsing
- **Standard flags**: Support `--help` and `--version`
- **Streams**: `stdout` for program output, `stderr` for errors and diagnostics, `stdin` for piped input
- **Signals**: Handle `SIGINT` and `SIGTERM` for graceful shutdown (flush buffers, release resources)
