# Dawn

Cross-platform task management application.  
Dawn stores tasks in a local-first SQLite database.  
Multiple clients (desktop CLI, iOS app) sync through a self-hosted central server.

## Architecture

### One hexagon, many clients

Dawn is a **single hexagon**. `core/` owns the domain and the driven adapters; every
client is a driving adapter on top of it.  
Layer directories are named `inbound/`, `domain/`, `outbound/`.

- `core/domain/` — entities, value objects, domain errors, and the ports (traits) the
  domain owns. No I/O, no SQL, no clap, no chrono formatting concerns leaking in.
- `core/outbound/` — driven adapters: SQLite repositories.
- Inbound adapters live with their client:
  - `cli/` — clap parsing, command handlers, table output.
  - `ios/` — SwiftUI views and view models, calling `core` across FFI.
  - `server/` — HTTP handlers. The server also carries its own `outbound/` for what
    core does not provide (multi-device reconciliation storage, auth).

  The one exception is `core/ffi/`. uniffi's Swift generator reads the *built* core
  library, so that adapter has to ship inside the core crate rather than in `ios/`.
  It holds binding declarations only — no domain logic, no decisions.

Dependency direction is always inward: `inbound → domain ← outbound`.
The domain never imports from `inbound/` or `outbound/`.

### Sync

- Central server hosted on a Raspberry Pi 4, deployable via Docker
  (must build for `linux/arm64`).
- Clients work fully offline; sync is an explicit or background reconciliation step.

## CLI: Taskwarrior parity

The CLI deliberately follows Taskwarrior's UX. Treat Taskwarrior as the spec.

- Reference source checkout: `~/Downloads/taskwarrior` (currently at **v2.6.2**).
- Installed binary for behavior checks: `task` (currently **3.5.0**). The source and the
  binary are different major versions — the binary's behavior wins.
- **Verify parity claims by running the real `task` binary.** Reading the C++ source is
  not sufficient — edge cases are easy to misread. Run it, capture the output, compare.
- **Domain error strings copy Taskwarrior verbatim**, including capitalization and
  trailing periods. Do not "fix" them to idiomatic Rust lowercase/no-period style.
- Choose the parser by Taskwarrior's *capability* flags, not by command intent:
  `mods=Y` → mutation parser, `mods=N` → report parser.

## Build & test

### Rust

```sh
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace -- -D warnings
cargo +nightly fmt
```

### iOS

```sh
xcodebuild -project ios/Dawn.xcodeproj -scheme Dawn -destination 'generic/platform=iOS' build
xcodebuild -project ios/Dawn.xcodeproj -scheme Dawn -destination 'platform=iOS Simulator,name=iPhone 17' test
```

## Version control

This repo is **jujutsu (`jj`) colocated with git**. `git status` reporting
"Not currently on any branch" (detached HEAD) is normal and expected — jj drives HEAD.

- Inspect history with `jj log`; `git log` also works read-only.
- Prefer `jj` commands for commits unless asked otherwise.
