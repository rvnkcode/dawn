# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dawn is a task management application being rewritten in Rust (RIIR - Rewrite It In Rust), inspired by Taskwarrior.
The project is designed with Hexagonal Architecture principles and follows a monorepo structure using Cargo workspaces.

## Taskwarrior Reference

When implementing features, refer to the original Taskwarrior for behavior and functionality:

- **Source Code**: `~/Downloads/taskwarrior` (C++ implementation)
- **Documentation**: <https://taskwarrior.org/docs/>
- **Man Pages**: `man task` for command reference
- **Man Pages Source**: `~/Downloads/taskwarrior/doc/man/` (detailed documentation)
- **Reference PDF**: `~/Downloads/taskwarrior/doc/ref/task-ref.pdf` (comprehensive reference)

### Implementation Verification

You can verify Dawn's implementation by comparing with Taskwarrior:

```bash
# Run original taskwarrior
task add "Test task"
task list

# Compare with dawn implementation
cargo run -p dawn_cli -- add "Test task"
cargo run -p dawn_cli -- list
```

This side-by-side comparison helps ensure feature parity and correct behavior.

## Hexagonal Architecture Reference

The project follows Hexagonal Architecture (Ports and Adapters) pattern:

- **Reference Implementation**: `~/Downloads/hexarch` (Rust example)
- **Architecture Guide**: <https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust>

Key principles to follow:

- Domain layer has no dependencies on infrastructure or presentation
- Use Port traits to define boundaries
- Adapters implement ports (inbound for UI/CLI, outbound for DB/external services)

## Architecture

### Workspace Structure

This is a **Cargo workspace monorepo** with the following organization:

```
dawn/
├── libs/dawn/          # Core library (domain logic + outbound adapters)
└── inbounds/cli/       # CLI binary application
```

**Design Pattern**: Hexagonal Architecture (Ports and Adapters)

- `libs/dawn/`: Contains domain logic and outbound adapters (infrastructure)
- `inbounds/`: Contains inbound adapters (user-facing applications like CLI, and future GUI)

**Key Architectural Decisions**:

1. **Dependency Separation**: Binary crates (CLI/GUI) import the core library, never the reverse
2. **Shared Business Logic**: All business logic lives in `libs/dawn` and is shared by all inbound applications
3. **Multiple Binaries**: The workspace supports multiple binaries (CLI now, GUI planned) that depend on the same core library

### Dependency Flow

```
inbounds/cli (binary) → libs/dawn (library)
inbounds/gui (future)  → libs/dawn (library)
```

The core library (`libs/dawn`) has minimal dependencies and contains:

- Domain entities, value objects, and services
- Port trait definitions (interfaces)
- Outbound adapter implementations (SQLite, metrics, etc.)

Binary crates add their specific dependencies (e.g., `clap` for CLI, `tauri` for GUI).

## Development Principles

When implementing features or reviewing code, ensure the following:

### 1. Hexagonal Architecture Compliance

- Domain layer (`libs/dawn/domain/`) must have NO dependencies on infrastructure or presentation
- All external interactions go through Port traits (interfaces)
- Adapters implement ports, never the reverse
- Dependency direction: Inbound/Outbound → Domain (never Domain → Adapters)

### 2. Object-Oriented Design (OOP)

- **Encapsulation**: Domain entities hide internal state, expose behavior through methods
- **Abstraction**: Use traits (ports) to define contracts
- **Polymorphism**: Generic implementations over trait bounds
- **Composition**: Prefer composition over inheritance (Rust idiom)

### 3. Domain-Driven Design (DDD)

- Rich domain models with business logic in entities and services
- Value objects for validated data (e.g., `TaskId`, `Description`)
- Entities with identity (e.g., `Task`)
- Domain services for operations that don't belong to a single entity
- Ubiquitous language: Use Taskwarrior terminology

### 4. Agile Development

- **Small commits**: Each commit should represent a single logical change
- **Incremental implementation**: Build features in small, testable increments
- **Iterative refinement**: Start simple, refactor as understanding grows
- Commit messages should clearly describe what and why

### 5. Testing

- **Unit tests**: For domain logic (entities, value objects, services)
- **Integration tests**: For adapters (repository implementations, etc.)
- **Test coverage**: All business logic must have tests
- **TDD encouraged**: Write tests before implementation when possible

### 6. POSIX Compliance (CLI)

- Follow POSIX standards for command-line interface where possible
- Use standard option formats: `-s` for short, `--long` for long options
- Support `--help` and `--version`
- Return appropriate exit codes (0 for success, non-zero for errors)
- Respect standard streams (stdin, stdout, stderr)
- Support common patterns like `--` to separate options from arguments

## Development Commands

### Building

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p dawn        # Library only
cargo build -p dawn_cli    # CLI only

# Release build
cargo build --release -p dawn_cli
```

### Running

```bash
# Run CLI
cargo run -p dawn_cli

# Run with arguments
cargo run -p dawn_cli -- [args]
```

### Testing

```bash
# Test entire workspace
cargo test

# Test specific crate
cargo test -p dawn
cargo test -p dawn_cli
```

### Checking

```bash
# Check all code without building
cargo check

# Check specific package
cargo check -p dawn
```

## Workspace Configuration

- **Resolver**: Uses Cargo resolver v3 (latest, as recommended by Rust Book)
- **Edition**: 2024
- **Lockfile**: `Cargo.lock` is committed to version control (required for binary projects)
- **Workspace Dependencies**: Defined in root `Cargo.toml` under `[workspace.dependencies]` for version consistency

## Future Expansion

The architecture is prepared for:

- **GUI Application**: Will be added as `inbounds/gui/`
- **Additional Libraries**: Can be added under `libs/` as needed
- **Microservices**: The hexagonal architecture allows easy extraction of services if needed

When adding the GUI:

1. Create `inbounds/gui/` with its own `Cargo.toml`
2. Add to workspace members in root `Cargo.toml`
3. Import `dawn = { workspace = true }` to access core library
4. Add GUI-specific dependencies (e.g., `tauri`) only in GUI crate
