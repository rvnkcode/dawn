# Development Principles

When implementing features or reviewing code, ensure the following:

## Hexagonal Architecture Compliance

- Domain layer must have NO dependencies on infrastructure or presentation
- All external interactions go through Port traits (interfaces)
- Adapters implement ports, never the reverse
- Dependency direction: Inbound/Outbound → Domain (never Domain → Adapters)

## Object-Oriented Design (OOP)

- **Encapsulation**: Domain entities hide internal state, expose behavior through methods
- **Abstraction**: Use traits (ports) to define contracts
- **Polymorphism**: Generic implementations over trait bounds
- **Composition**: Prefer composition over inheritance (Rust idiom)

## Domain-Driven Design (DDD)

- Rich domain models with business logic in entities and services
- Value objects for validated data (e.g., `TaskId`, `Description`)
- Entities with identity (e.g., `Task`)
- Domain services for operations that don't belong to a single entity
- Ubiquitous language: Use Taskwarrior terminology

## Agile Development

- **Small commits**: Each commit should represent a single logical change
- **Incremental implementation**: Build features in small, testable increments
- **Iterative refinement**: Start simple, refactor as understanding grows
- Commit messages should clearly describe what and why

## Testing

- **Unit tests**: For domain logic (entities, value objects, services)
- **Integration tests**: For adapters (repository implementations, etc.)
- **Test coverage**: All business logic must have tests
- **TDD encouraged**: Write tests before implementation when possible

## POSIX Compliance (CLI)

- Follow POSIX standards for command-line interface where possible
- Use standard option formats: `-s` for short, `--long` for long options
- Support `--help` and `--version`
- Return appropriate exit codes (0 for success, non-zero for errors)
- Respect standard streams (stdin, stdout, stderr)
- Support common patterns like `--` to separate options from arguments
