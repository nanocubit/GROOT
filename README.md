# GROOT

GROOT is an experimental integration workspace for exploring the migration path from grit to a gitoxide-backed backend.

This repository is intentionally small and focused. It is not meant to be a full standalone Git implementation. Instead, GROOT exists to test, validate, and structure the architecture needed to connect:

- grit: Git-compatible engine with CLI semantics and edge-case behavior
- gitoxide: mature Rust backend ecosystem for Git plumbing

## Why GROOT exists

The current grit architecture already shows a clear separation between:

- low-level Git behaviors
- repository discovery and environment semantics
- object access and index logic
- worktree and repo-specific edge cases
- CLI/user-facing commands

At the same time, gitoxide already provides a stronger Rust backend foundation:

- gix-discover
- gix-odb
- gix-index
- gix-worktree
- gix-ref
- gix-transport

The goal of GROOT is to create the migration layer needed to connect these two systems without rewriting everything at once.

## Core idea

GROOT is a compatibility and adapter workspace.

It is designed to answer one question:

Can grit be migrated gradually to a gitoxide-backed backend while preserving Git-compatible behavior and user experience?

The answer is yes, through an explicit adaptation layer.

## Architecture

The initial design is intentionally incremental:

1. backend abstraction
2. legacy fallback
3. gitoxide adapter skeleton
4. compatibility smoke tests
5. migration of discovery / object / index behavior
6. later migration of refs / transport / CLI layer

## Primary goals

- keep grit behavior as the compatibility reference
- preserve a fallback path to legacy grit logic
- introduce a backend trait/interface
- test gitoxide as the backend for repository discovery and object/index access
- keep the migration low-risk and incremental

## Non-goals

GROOT is not intended to become:

- a competing full Git implementation
- a rewrite of all of grit in one pass
- a monolithic backend without compatibility tests

This project should remain:

- small
- explicit
- migration-oriented
- compatibility-focused

## Repository structure

```text
GROOT/
├── Cargo.toml
├── README.md
├── .github/
│   └── workflows/
│       └── ci.yml
├── docs/
│   └── migration_plan.md
├── crates/
│   ├── groot-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backend_interface.rs
│   │       └── adapters/
│   │           ├── legacy.rs
│   │           └── gitoxide.rs
│   └── groot-compat/
│       ├── Cargo.toml
│       └── tests/
│           └── discovery_and_basic.rs
└── ...
```

## Migration strategy

The practical migration order is:

1. Introduce a backend abstraction
2. Keep legacy behavior as fallback
3. Wire gitoxide adapter for discovery
4. Migrate object database access
5. Migrate index access
6. Migrate refs and worktree semantics
7. Migrate transport and remote logic
8. Keep grit CLI semantics stable

## Compatibility targets

The first compatibility targets are derived from the current grit architecture and behavior:

- repository discovery from nested directories
- `.git` file vs `.git` directory behavior
- GIT_DIR / GIT_WORK_TREE overrides
- linked worktree semantics
- bare repository detection
- index roundtrip behavior
- object access from loose / packed stores

## Status

This repository is currently an integration scaffold and migration sandbox. It is intentionally minimal and designed to support the next stage of actual backend migration.

## License

MIT

## Related projects

- grit
- gitoxide
